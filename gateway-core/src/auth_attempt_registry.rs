//! Server-owned registry for short-lived browser authentication attempts.
//!
//! This module owns the opaque lookup identity and lifecycle of
//! [`BrowserAuthAttempt`] values.  It deliberately exposes only the bounded
//! `BrowserAuthEvent` stream; worker events, profile capabilities, Vault
//! material, and site-specific interpretation remain behind the runtime.

use crate::browser::BrowserWorker;
use crate::browser_auth::{
    BrowserAuthAttempt, BrowserAuthEvent, BrowserAuthRuntime, BrowserAuthRuntimeError,
    MAX_AUTH_EVENTS,
};
use site_adapter_api::BrowserAuthState;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Opaque server-owned identity for a browser authentication attempt.
///
/// The identifier is suitable for a later transport layer, but does not carry
/// a site, account, Vault reference, or browser session identity.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct BrowserAuthAttemptId(String);

impl BrowserAuthAttemptId {
    fn new() -> Self {
        Self(format!("auth-{}", Uuid::new_v4().simple()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for BrowserAuthAttemptId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BrowserAuthAttemptId([opaque])")
    }
}

impl fmt::Display for BrowserAuthAttemptId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Safe lifecycle information for a registered attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserAuthAttemptSnapshot {
    pub id: BrowserAuthAttemptId,
    pub state: BrowserAuthState,
    pub expires_in: Duration,
}

/// Registry errors have stable, secret-free categories for a future API
/// projection.  Runtime details remain server-side and are already bounded
/// by `BrowserAuthRuntimeError`.
#[derive(Debug)]
pub enum BrowserAuthRegistryError {
    AttemptNotFound,
    AttemptExpired,
    Runtime(BrowserAuthRuntimeError),
}

impl BrowserAuthRegistryError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::AttemptNotFound => "AUTH_ATTEMPT_NOT_FOUND",
            Self::AttemptExpired => "AUTH_ATTEMPT_EXPIRED",
            Self::Runtime(error) => match error {
                BrowserAuthRuntimeError::Browser(error) => error.code(),
                BrowserAuthRuntimeError::Vault(_) => "AUTH_VAULT_REJECTED",
                BrowserAuthRuntimeError::CandidateRejected => "AUTH_CANDIDATE_REJECTED",
                BrowserAuthRuntimeError::InvalidCandidate => "AUTH_CANDIDATE_INVALID",
                BrowserAuthRuntimeError::InvalidObservation => "AUTH_OBSERVATION_INVALID",
                BrowserAuthRuntimeError::CandidateCaptureRequestMismatch => {
                    "AUTH_CANDIDATE_CAPTURE_REQUEST_MISMATCH"
                }
                BrowserAuthRuntimeError::CandidateCaptureAlreadyConsumed => {
                    "AUTH_CANDIDATE_CAPTURE_ALREADY_CONSUMED"
                }
            },
        }
    }
}

impl fmt::Display for BrowserAuthRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for BrowserAuthRegistryError {}

impl From<BrowserAuthRuntimeError> for BrowserAuthRegistryError {
    fn from(error: BrowserAuthRuntimeError) -> Self {
        Self::Runtime(error)
    }
}

struct RegisteredAttempt<W: BrowserWorker> {
    id: BrowserAuthAttemptId,
    attempt: BrowserAuthAttempt<W>,
}

/// Server-owned lifecycle registry around the generic browser auth runtime.
///
/// The registry is intentionally transport-neutral.  A future HTTP/control
/// layer can use the opaque ID and safe snapshots/events without owning the
/// runtime, worker, or Vault state itself.
pub struct BrowserAuthAttemptRegistry<W: BrowserWorker + 'static> {
    runtime: BrowserAuthRuntime<W>,
    attempts: Arc<Mutex<HashMap<BrowserAuthAttemptId, RegisteredAttempt<W>>>>,
}

impl<W: BrowserWorker + 'static> Clone for BrowserAuthAttemptRegistry<W> {
    fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            attempts: Arc::clone(&self.attempts),
        }
    }
}

impl<W: BrowserWorker + 'static> BrowserAuthAttemptRegistry<W> {
    pub fn new(runtime: BrowserAuthRuntime<W>) -> Self {
        Self {
            runtime,
            attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start one short-lived server-owned attempt and return only its opaque
    /// handle plus safe lifecycle information.
    pub async fn start(
        &self,
        site_id: impl Into<String>,
        account_ref: impl Into<String>,
    ) -> Result<BrowserAuthAttemptSnapshot, BrowserAuthRegistryError> {
        let attempt = self.runtime.start(site_id, account_ref).await?;
        let id = BrowserAuthAttemptId::new();
        let snapshot = snapshot(&id, &attempt);
        self.attempts
            .lock()
            .expect("browser auth attempt registry poisoned")
            .insert(id.clone(), RegisteredAttempt { id, attempt });
        Ok(snapshot)
    }

    /// Return safe lifecycle information, lazily removing expired attempts.
    pub fn snapshot(
        &self,
        id: &BrowserAuthAttemptId,
    ) -> Result<BrowserAuthAttemptSnapshot, BrowserAuthRegistryError> {
        let mut registered = self.take(id)?;
        if registered.attempt.is_expired() {
            registered.attempt.expire();
            return Err(BrowserAuthRegistryError::AttemptExpired);
        }
        let snapshot = snapshot(&registered.id, &registered.attempt);
        self.put(registered);
        Ok(snapshot)
    }

    /// Read only bounded, redacted auth observations after a cursor.  The
    /// runtime keeps a bounded ring; the registry applies the same bound at
    /// its transport-neutral boundary.
    pub fn events_after(
        &self,
        id: &BrowserAuthAttemptId,
        after_sequence: u64,
    ) -> Result<Vec<BrowserAuthEvent>, BrowserAuthRegistryError> {
        let mut registered = self.take(id)?;
        if registered.attempt.is_expired() {
            registered.attempt.expire();
            return Err(BrowserAuthRegistryError::AttemptExpired);
        }
        let events = registered
            .attempt
            .events_after(after_sequence)
            .into_iter()
            .take(MAX_AUTH_EVENTS)
            .collect();
        self.put(registered);
        Ok(events)
    }

    /// Cancel and remove an attempt after closing its worker session.
    pub fn cancel(
        &self,
        id: &BrowserAuthAttemptId,
    ) -> Result<BrowserAuthAttemptSnapshot, BrowserAuthRegistryError> {
        let mut registered = self.take(id)?;
        if registered.attempt.is_expired() {
            registered.attempt.expire();
            return Err(BrowserAuthRegistryError::AttemptExpired);
        }
        registered.attempt.cancel()?;
        Ok(snapshot(&registered.id, &registered.attempt))
    }

    /// Close and remove every attempt whose TTL has elapsed.  This is
    /// explicit so a deployment can call it from its own maintenance loop;
    /// reads and cancellation also perform lazy cleanup.
    pub fn cleanup_expired(&self) -> usize {
        let expired = {
            let mut attempts = self
                .attempts
                .lock()
                .expect("browser auth attempt registry poisoned");
            let ids = attempts
                .iter()
                .filter(|(_, registered)| registered.attempt.is_expired())
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>();
            ids.into_iter()
                .filter_map(|id| attempts.remove(&id))
                .collect::<Vec<_>>()
        };
        let count = expired.len();
        for mut registered in expired {
            registered.attempt.expire();
        }
        count
    }

    pub fn len(&self) -> usize {
        self.attempts
            .lock()
            .expect("browser auth attempt registry poisoned")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn take(
        &self,
        id: &BrowserAuthAttemptId,
    ) -> Result<RegisteredAttempt<W>, BrowserAuthRegistryError> {
        self.attempts
            .lock()
            .expect("browser auth attempt registry poisoned")
            .remove(id)
            .ok_or(BrowserAuthRegistryError::AttemptNotFound)
    }

    fn put(&self, registered: RegisteredAttempt<W>) {
        self.attempts
            .lock()
            .expect("browser auth attempt registry poisoned")
            .insert(registered.id.clone(), registered);
    }
}

fn snapshot<W: BrowserWorker>(
    id: &BrowserAuthAttemptId,
    attempt: &BrowserAuthAttempt<W>,
) -> BrowserAuthAttemptSnapshot {
    BrowserAuthAttemptSnapshot {
        id: id.clone(),
        state: attempt.state(),
        expires_in: attempt.expires_in(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::SessionVault;
    use crate::browser::FakeBrowserWorker;
    use crate::browser_auth::{BrowserAuthRuntime, MAX_AUTH_EVENTS};
    use site_adapter_api::BrowserAuthState;
    use std::time::Duration;

    const SITE: &str = "site-a";
    const ACCOUNT: &str = "account-a";

    fn registry(ttl: Duration) -> BrowserAuthAttemptRegistry<FakeBrowserWorker> {
        let vault = SessionVault::isolated_test();
        vault.register_account(SITE, ACCOUNT, "fixture").unwrap();
        BrowserAuthAttemptRegistry::new(BrowserAuthRuntime::with_ttl(
            FakeBrowserWorker::new(),
            vault,
            ttl,
        ))
    }

    #[tokio::test]
    async fn starts_with_opaque_id_and_bounded_redacted_events() {
        let registry = registry(Duration::from_secs(30));
        let started = registry.start(SITE, ACCOUNT).await.unwrap();
        assert!(started.id.as_str().starts_with("auth-"));
        assert_eq!(started.state, BrowserAuthState::Required);
        let events = registry.events_after(&started.id, 0).unwrap();
        assert!(!events.is_empty());
        assert!(events.len() <= MAX_AUTH_EVENTS);
        assert!(!format!("{events:?}").contains("cookie"));
        assert!(!format!("{started:?}").contains(SITE));
    }

    #[tokio::test]
    async fn cancel_closes_and_removes_attempt() {
        let registry = registry(Duration::from_secs(30));
        let started = registry.start(SITE, ACCOUNT).await.unwrap();
        let cancelled = registry.cancel(&started.id).unwrap();
        assert_eq!(cancelled.state, BrowserAuthState::Cancelled);
        assert_eq!(registry.len(), 0);
        assert!(matches!(
            registry.events_after(&started.id, 0),
            Err(BrowserAuthRegistryError::AttemptNotFound)
        ));
    }

    #[tokio::test]
    async fn expired_attempt_is_closed_and_removed() {
        let registry = registry(Duration::ZERO);
        let started = registry.start(SITE, ACCOUNT).await.unwrap();
        assert!(matches!(
            registry.snapshot(&started.id),
            Err(BrowserAuthRegistryError::AttemptExpired)
        ));
        assert_eq!(registry.len(), 0);
        assert_eq!(registry.cleanup_expired(), 0);
    }

    #[tokio::test]
    async fn explicit_expiry_cleanup_removes_all_expired_attempts() {
        let registry = registry(Duration::ZERO);
        registry.start(SITE, ACCOUNT).await.unwrap();
        registry.start(SITE, ACCOUNT).await.unwrap();
        assert_eq!(registry.cleanup_expired(), 2);
        assert_eq!(registry.len(), 0);
    }
}
