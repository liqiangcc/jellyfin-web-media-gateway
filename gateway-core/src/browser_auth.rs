//! Vault-bound authenticated Browser Worker orchestration.
//!
//! This module composes the generic Browser Worker with the Session Vault. It
//! owns only lifecycle and capability plumbing: site-specific login meaning is
//! interpreted by the Site Plugin, while PlaybackSession and Display remain
//! outside this state machine.

use crate::auth::{CandidateValidation, SessionVault, SiteSessionRef, VaultError};
use crate::browser::{
    BrowserAuthMode, BrowserError, BrowserEvent, BrowserNavigationRequest,
    BrowserObservationHandoff, BrowserOperationId, BrowserSession, BrowserWorker,
    R008NavigationPolicy,
};
use site_adapter_api::{
    AuthenticatedSessionHandoff, BROWSER_AUTH_OBSERVATION_VERSION, BrowserAuthDiagnostic,
    BrowserAuthObservation, BrowserAuthState, SourceLocator, validate_browser_auth_observation,
};
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const AUTH_EVENT_VERSION: u16 = 1;
pub const MAX_AUTH_EVENTS: usize = 64;
pub const AUTH_ATTEMPT_TTL: Duration = Duration::from_secs(300);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrowserAuthRuntimeError {
    Browser(BrowserError),
    Vault(VaultError),
    CandidateRejected,
    InvalidCandidate,
    InvalidObservation,
    CandidateCaptureRequestMismatch,
    CandidateCaptureAlreadyConsumed,
}

impl fmt::Display for BrowserAuthRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Browser(error) => error.fmt(formatter),
            Self::Vault(error) => write!(formatter, "vault error: {error:?}"),
            Self::CandidateRejected => formatter.write_str("CANDIDATE_REJECTED"),
            Self::InvalidCandidate => formatter.write_str("INVALID_CANDIDATE"),
            Self::InvalidObservation => formatter.write_str("INVALID_OBSERVATION"),
            Self::CandidateCaptureRequestMismatch => {
                formatter.write_str("CANDIDATE_CAPTURE_REQUEST_MISMATCH")
            }
            Self::CandidateCaptureAlreadyConsumed => {
                formatter.write_str("CANDIDATE_CAPTURE_ALREADY_CONSUMED")
            }
        }
    }
}

impl std::error::Error for BrowserAuthRuntimeError {}

impl From<BrowserError> for BrowserAuthRuntimeError {
    fn from(error: BrowserError) -> Self {
        Self::Browser(error)
    }
}

impl From<VaultError> for BrowserAuthRuntimeError {
    fn from(error: VaultError) -> Self {
        Self::Vault(error)
    }
}

/// A bounded generic auth event. Its observation contains only enum state and
/// a fixed diagnostic code, so passwords, codes, URLs and profile material
/// cannot enter the event stream accidentally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrowserAuthEvent {
    pub version: u16,
    pub observation: BrowserAuthObservation,
}

pub struct BrowserAuthRuntime<W: BrowserWorker> {
    worker: Arc<W>,
    vault: SessionVault,
    ttl: Duration,
}

impl<W: BrowserWorker> Clone for BrowserAuthRuntime<W> {
    fn clone(&self) -> Self {
        Self {
            worker: Arc::clone(&self.worker),
            vault: self.vault.clone(),
            ttl: self.ttl,
        }
    }
}

impl<W: BrowserWorker + 'static> BrowserAuthRuntime<W> {
    pub fn new(worker: W, vault: SessionVault) -> Self {
        Self {
            worker: Arc::new(worker),
            vault,
            ttl: AUTH_ATTEMPT_TTL,
        }
    }

    pub fn with_ttl(worker: W, vault: SessionVault, ttl: Duration) -> Self {
        Self {
            ttl,
            ..Self::new(worker, vault)
        }
    }

    /// Open an Interactive session with a fresh worker profile. If an active
    /// Vault session has a browser profile, the runtime receives a one-shot
    /// opaque attachment capability and materializes it into that fresh
    /// disposable profile. Accounts without an existing session still get a
    /// fresh profile and can proceed through the later candidate flow.
    pub async fn start(
        &self,
        site_id: impl Into<String>,
        account_ref: impl Into<String>,
    ) -> Result<BrowserAuthAttempt<W>, BrowserAuthRuntimeError> {
        let site_id = site_id.into();
        let account_ref = account_ref.into();
        let active = self.vault.active_session(&site_id, &account_ref)?;
        let session = self
            .worker
            .open_session(BrowserAuthMode::Interactive)
            .await?;

        if let Some(active) = active {
            let attachment = match self.vault.issue_profile_attachment_ref(&active) {
                Ok(attachment) => attachment,
                Err(error) => {
                    let _ = self.worker.close(session.id());
                    return Err(error.into());
                }
            };
            let materializer = self.vault.profile_materializer();
            if let Err(error) = self
                .worker
                .attach_profile_with_materializer(session.id(), attachment, materializer)
                .await
            {
                let _ = self.worker.close(session.id());
                return Err(error.into());
            }
        }

        let mut attempt = BrowserAuthAttempt {
            worker: Arc::clone(&self.worker),
            vault: self.vault.clone(),
            session,
            site_id,
            account_ref,
            expires_at: Instant::now() + self.ttl,
            state: BrowserAuthState::Required,
            events: VecDeque::new(),
            next_sequence: 0,
            closed: false,
            candidate_capture: None,
            committed_candidate: None,
        };
        attempt.emit(BrowserAuthState::Required, BrowserAuthDiagnostic::None);
        Ok(attempt)
    }
}

pub struct BrowserAuthAttempt<W: BrowserWorker> {
    worker: Arc<W>,
    vault: SessionVault,
    session: BrowserSession,
    site_id: String,
    account_ref: String,
    expires_at: Instant,
    state: BrowserAuthState,
    events: VecDeque<BrowserAuthEvent>,
    next_sequence: u64,
    closed: bool,
    candidate_capture: Option<CandidateCaptureRecord>,
    committed_candidate: Option<SiteSessionRef>,
}

#[derive(Clone)]
struct CandidateCaptureRecord {
    request_id: String,
    observation: BrowserAuthObservation,
    candidate: SiteSessionRef,
}

impl<W: BrowserWorker> fmt::Debug for BrowserAuthAttempt<W> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserAuthAttempt")
            .field("session", &self.session)
            .field("site_id", &self.site_id)
            .field("account_ref", &"[opaque]")
            .field("expires_at", &"[short-lived]")
            .field("state", &self.state)
            .field("events", &self.events)
            .finish()
    }
}

impl<W: BrowserWorker> BrowserAuthAttempt<W> {
    pub fn session(&self) -> &BrowserSession {
        &self.session
    }

    pub fn site_id(&self) -> &str {
        &self.site_id
    }

    pub fn account_ref(&self) -> &str {
        &self.account_ref
    }

    pub fn state(&self) -> BrowserAuthState {
        self.state
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= Instant::now()
    }

    pub fn expires_in(&self) -> Duration {
        self.expires_at.saturating_duration_since(Instant::now())
    }

    pub fn events_after(&self, after_sequence: u64) -> Vec<BrowserAuthEvent> {
        self.events
            .iter()
            .copied()
            .filter(|event| event.observation.sequence > after_sequence)
            .collect()
    }

    pub fn last_observation(&self) -> Option<BrowserAuthObservation> {
        self.events.back().map(|event| event.observation)
    }

    /// Return only the generic worker events retained for this session. The
    /// worker's event contract already redacts input, URLs and profile data;
    /// auth interpretation remains a Site Plugin responsibility.
    pub fn browser_events_after(
        &mut self,
        after_sequence: u64,
    ) -> Result<Vec<BrowserEvent>, BrowserAuthRuntimeError> {
        self.ensure_live()?;
        self.worker
            .poll_events(self.session.id(), after_sequence)
            .map_err(Into::into)
    }

    pub async fn navigate(
        &mut self,
        request: BrowserNavigationRequest,
        policy: &R008NavigationPolicy,
    ) -> Result<BrowserOperationId, BrowserAuthRuntimeError> {
        self.ensure_live()?;
        let operation_id = request.operation_id();
        self.worker
            .navigate(self.session.id(), request, policy)
            .await
            .map_err(|error| {
                self.observe_worker_error(error);
                error.into()
            })
            .map(|()| operation_id)
    }

    pub async fn request_input(
        &mut self,
        input: crate::browser::BrowserInput,
    ) -> Result<(), BrowserAuthRuntimeError> {
        self.ensure_live()?;
        self.emit(
            BrowserAuthState::InputNeeded,
            BrowserAuthDiagnostic::AwaitingInput,
        );
        self.worker
            .send_input(self.session.id(), input)
            .await
            .map_err(|error| {
                self.observe_worker_error(error);
                error.into()
            })
    }

    pub fn cancel_operation(
        &mut self,
        operation_id: BrowserOperationId,
    ) -> Result<(), BrowserAuthRuntimeError> {
        self.ensure_live()?;
        self.worker
            .cancel(self.session.id(), operation_id)
            .map_err(Into::into)
    }

    /// Consume the worker's one-shot generic observation and bind it to the
    /// opaque locator selected by the owning SiteAdapterRegistry.
    pub fn take_observation_handoff(
        &mut self,
        locator: SourceLocator,
        operation_id: BrowserOperationId,
        ttl: Duration,
    ) -> Result<BrowserObservationHandoff, BrowserAuthRuntimeError> {
        self.ensure_live()?;
        BrowserObservationHandoff::take_from_worker(
            self.worker.as_ref(),
            self.session.id(),
            operation_id,
            locator,
            ttl,
        )?
        .ok_or(BrowserAuthRuntimeError::InvalidObservation)
    }

    /// Capture the worker-owned authenticated material exactly once for this
    /// live attempt. The Vault receives the material through a crate-private
    /// seam and returns only an opaque candidate session reference.
    pub async fn capture_candidate(
        &mut self,
        request_id: &str,
        observation: BrowserAuthObservation,
    ) -> Result<SiteSessionRef, BrowserAuthRuntimeError> {
        self.ensure_live()?;
        if request_id.is_empty()
            || request_id.len() > 128
            || request_id.chars().any(char::is_control)
        {
            return Err(BrowserAuthRuntimeError::InvalidCandidate);
        }
        validate_browser_auth_observation(&observation)
            .map_err(|_| BrowserAuthRuntimeError::InvalidObservation)?;
        if observation.state != BrowserAuthState::CandidateReady {
            return Err(BrowserAuthRuntimeError::InvalidCandidate);
        }
        if let Some(previous) = self.candidate_capture.as_ref() {
            if previous.request_id == request_id && previous.observation == observation {
                return Ok(previous.candidate.clone());
            }
            return Err(BrowserAuthRuntimeError::CandidateCaptureRequestMismatch);
        }
        if self.committed_candidate.is_some() {
            return Err(BrowserAuthRuntimeError::CandidateCaptureAlreadyConsumed);
        }

        let material = match self
            .worker
            .capture_candidate_material(self.session.id())
            .await
        {
            Ok(material) => material,
            Err(error) => {
                self.observe_worker_error(error);
                return Err(error.into());
            }
        };
        let candidate =
            self.vault
                .capture_candidate_material(&self.site_id, &self.account_ref, material)?;
        self.candidate_capture = Some(CandidateCaptureRecord {
            request_id: request_id.to_owned(),
            observation,
            candidate: candidate.clone(),
        });
        self.emit(
            BrowserAuthState::CandidateReady,
            BrowserAuthDiagnostic::CandidateAccepted,
        );
        Ok(candidate)
    }

    /// Resolve an opaque candidate identity through the Vault and perform the
    /// existing atomic validation/swap.  Raw session material never leaves
    /// this runtime.
    pub fn accept_candidate_ref(
        &mut self,
        candidate_session_id: &str,
        observation: BrowserAuthObservation,
    ) -> Result<AuthenticatedSessionHandoff, BrowserAuthRuntimeError> {
        let candidate = self.vault.candidate_session_ref(
            &self.site_id,
            &self.account_ref,
            candidate_session_id,
        )?;
        if let Some(captured) = self.candidate_capture.as_ref()
            && captured.candidate != candidate
        {
            return Err(BrowserAuthRuntimeError::InvalidCandidate);
        }
        self.accept_candidate(candidate, observation)
    }

    /// Record a generic candidate-ready observation after the Site Plugin has
    /// interpreted the Browser Worker facts. The runtime validates only the
    /// generic schema and candidate identity; it never decides site semantics.
    pub fn accept_candidate(
        &mut self,
        candidate: SiteSessionRef,
        observation: BrowserAuthObservation,
    ) -> Result<AuthenticatedSessionHandoff, BrowserAuthRuntimeError> {
        self.ensure_live()?;
        let captured_mismatch = self.candidate_capture.as_ref().is_some_and(|captured| {
            captured.candidate != candidate || captured.observation != observation
        });
        if captured_mismatch {
            self.reject_candidate_if_present(&candidate);
            return Err(BrowserAuthRuntimeError::InvalidCandidate);
        }
        validate_browser_auth_observation(&observation).map_err(|_| {
            self.reject_candidate_if_present(&candidate);
            BrowserAuthRuntimeError::InvalidObservation
        })?;
        if observation.state != BrowserAuthState::CandidateReady
            || candidate.site_id() != self.site_id
            || candidate.account_ref() != self.account_ref
        {
            self.reject_candidate_if_present(&candidate);
            self.emit(
                BrowserAuthState::CandidateReady,
                BrowserAuthDiagnostic::CandidateRejected,
            );
            return Err(BrowserAuthRuntimeError::InvalidCandidate);
        }
        // Construct the opaque handoff before mutating the Vault. If the
        // generic handoff contract rejects malformed metadata, the prior
        // active session must remain untouched.
        let handoff = AuthenticatedSessionHandoff::new_server_owned(
            self.site_id.clone(),
            self.account_ref.clone(),
            candidate.session_id().to_owned(),
            observation,
        )
        .map_err(|_| {
            self.reject_candidate_if_present(&candidate);
            BrowserAuthRuntimeError::InvalidObservation
        })?;
        self.vault
            .validate_and_swap(&candidate, CandidateValidation::Valid)
            .map_err(|error| {
                self.emit(
                    BrowserAuthState::CandidateReady,
                    BrowserAuthDiagnostic::CandidateRejected,
                );
                if error == VaultError::CandidateRejected {
                    BrowserAuthRuntimeError::CandidateRejected
                } else {
                    error.into()
                }
            })?;
        self.emit(
            BrowserAuthState::CandidateReady,
            BrowserAuthDiagnostic::CandidateAccepted,
        );
        self.committed_candidate = Some(candidate.clone());
        Ok(handoff)
    }

    /// Validate and discard a candidate without touching the active session.
    pub fn reject_candidate(
        &mut self,
        candidate: &SiteSessionRef,
        observation: BrowserAuthObservation,
    ) -> Result<(), BrowserAuthRuntimeError> {
        self.ensure_live()?;
        validate_browser_auth_observation(&observation)
            .map_err(|_| BrowserAuthRuntimeError::InvalidObservation)?;
        if candidate.site_id() != self.site_id || candidate.account_ref() != self.account_ref {
            return Err(BrowserAuthRuntimeError::InvalidCandidate);
        }
        let _ = self
            .vault
            .validate_and_swap(candidate, CandidateValidation::Invalid);
        self.emit(
            BrowserAuthState::CandidateReady,
            BrowserAuthDiagnostic::CandidateRejected,
        );
        Err(BrowserAuthRuntimeError::CandidateRejected)
    }

    pub fn issue_site_access_capability(
        &mut self,
        session: &SiteSessionRef,
        allowed_hosts: impl IntoIterator<Item = String>,
        ttl: Duration,
    ) -> Result<crate::SiteAccessCapability, BrowserAuthRuntimeError> {
        self.ensure_live()?;
        if session.site_id() != self.site_id || session.account_ref() != self.account_ref {
            return Err(BrowserAuthRuntimeError::InvalidCandidate);
        }
        self.vault
            .issue_site_access_capability(session, allowed_hosts, ttl)
            .map_err(Into::into)
    }

    pub fn cancel(&mut self) -> Result<(), BrowserAuthRuntimeError> {
        self.ensure_live()?;
        self.emit(
            BrowserAuthState::Cancelled,
            BrowserAuthDiagnostic::Cancelled,
        );
        self.cleanup_worker();
        Ok(())
    }

    pub fn expire(&mut self) {
        if self.closed {
            return;
        }
        self.state = BrowserAuthState::Expired;
        self.emit(BrowserAuthState::Expired, BrowserAuthDiagnostic::Expired);
        self.cleanup_worker();
    }

    pub fn cleanup(&mut self) {
        if !self.closed {
            self.cleanup_worker();
            self.emit(
                BrowserAuthState::Cancelled,
                BrowserAuthDiagnostic::CleanupComplete,
            );
        }
    }

    fn ensure_live(&mut self) -> Result<(), BrowserAuthRuntimeError> {
        if self.closed {
            return Err(BrowserError::SessionClosed.into());
        }
        if self.is_expired() {
            self.expire();
            return Err(BrowserError::SessionExpired.into());
        }
        match self.worker.status(self.session.id()) {
            Ok(crate::browser::BrowserStatus::Open) => Ok(()),
            Ok(crate::browser::BrowserStatus::Crashed) => {
                self.observe_worker_error(BrowserError::WorkerCrashed);
                Err(BrowserError::WorkerCrashed.into())
            }
            Ok(crate::browser::BrowserStatus::TimedOut) => {
                self.observe_worker_error(BrowserError::WorkerTimeout);
                Err(BrowserError::WorkerTimeout.into())
            }
            Ok(crate::browser::BrowserStatus::Closed) => {
                self.observe_worker_error(BrowserError::SessionClosed);
                Err(BrowserError::SessionClosed.into())
            }
            Err(error) => Err(error.into()),
        }
    }

    fn observe_worker_error(&mut self, error: BrowserError) {
        let (state, diagnostic) = match error {
            BrowserError::WorkerCrashed => {
                (BrowserAuthState::Crashed, BrowserAuthDiagnostic::Crashed)
            }
            BrowserError::WorkerTimeout => {
                (BrowserAuthState::TimedOut, BrowserAuthDiagnostic::TimedOut)
            }
            BrowserError::SessionExpired => {
                (BrowserAuthState::Expired, BrowserAuthDiagnostic::Expired)
            }
            BrowserError::PanelDisconnected | BrowserError::WorkerUnavailable => (
                BrowserAuthState::Disconnected,
                BrowserAuthDiagnostic::Disconnected,
            ),
            BrowserError::OperationCancelled => (
                BrowserAuthState::Cancelled,
                BrowserAuthDiagnostic::Cancelled,
            ),
            BrowserError::SessionClosed => (
                BrowserAuthState::Disconnected,
                BrowserAuthDiagnostic::Disconnected,
            ),
            _ => return,
        };
        self.emit(state, diagnostic);
        self.discard_uncommitted_candidate();
        self.cleanup_worker();
    }

    fn reject_candidate_if_present(&mut self, candidate: &SiteSessionRef) {
        let _ = self.vault.cancel_candidate(candidate);
        if self
            .candidate_capture
            .as_ref()
            .is_some_and(|captured| captured.candidate == *candidate)
        {
            self.candidate_capture = None;
        }
    }

    fn discard_uncommitted_candidate(&mut self) {
        if self.committed_candidate.is_none()
            && let Some(captured) = self.candidate_capture.take()
        {
            self.reject_candidate_if_present(&captured.candidate);
        }
    }

    fn emit(&mut self, state: BrowserAuthState, diagnostic: BrowserAuthDiagnostic) {
        self.next_sequence += 1;
        self.state = state;
        if self.events.len() >= MAX_AUTH_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(BrowserAuthEvent {
            version: AUTH_EVENT_VERSION,
            observation: BrowserAuthObservation {
                schema_version: BROWSER_AUTH_OBSERVATION_VERSION,
                sequence: self.next_sequence,
                state,
                diagnostic,
            },
        });
    }

    fn cleanup_worker(&mut self) {
        if !self.closed {
            self.discard_uncommitted_candidate();
            let _ = self.worker.close(self.session.id());
            self.closed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CandidateValidation;
    use crate::browser::{BrowserCandidateMaterial, BrowserInput, FakeBrowserWorker};

    const SITE: &str = "site-a";
    const ACCOUNT: &str = "account-a";

    fn ready_observation() -> BrowserAuthObservation {
        BrowserAuthObservation {
            schema_version: BROWSER_AUTH_OBSERVATION_VERSION,
            sequence: 7,
            state: BrowserAuthState::CandidateReady,
            diagnostic: BrowserAuthDiagnostic::CandidateAccepted,
        }
    }

    #[tokio::test]
    async fn runtime_materializes_fresh_profile_and_consumes_attachment_once() {
        let vault = SessionVault::isolated_test();
        vault.register_account(SITE, ACCOUNT, "fixture").unwrap();
        let old = vault
            .create_fixture_candidate_session(SITE, ACCOUNT, "old")
            .unwrap();
        vault
            .validate_and_swap(&old, CandidateValidation::Valid)
            .unwrap();
        let runtime = BrowserAuthRuntime::with_ttl(
            FakeBrowserWorker::new(),
            vault.clone(),
            Duration::from_secs(30),
        );
        let mut attempt = runtime.start(SITE, ACCOUNT).await.unwrap();
        assert_eq!(attempt.state(), BrowserAuthState::Required);
        attempt
            .request_input(BrowserInput::Text {
                value: "password-must-not-appear".into(),
            })
            .await
            .unwrap();
        let diagnostics = format!("{attempt:?}");
        assert!(!diagnostics.contains("password-must-not-appear"));
        assert!(attempt.events_after(0).len() <= MAX_AUTH_EVENTS);
    }

    #[tokio::test]
    async fn invalid_candidate_preserves_active_session_and_valid_swap_is_atomic() {
        let vault = SessionVault::isolated_test();
        vault.register_account(SITE, ACCOUNT, "fixture").unwrap();
        let old = vault
            .create_fixture_candidate_session(SITE, ACCOUNT, "old")
            .unwrap();
        vault
            .validate_and_swap(&old, CandidateValidation::Valid)
            .unwrap();
        let runtime = BrowserAuthRuntime::new(FakeBrowserWorker::new(), vault.clone());
        let mut attempt = runtime.start(SITE, ACCOUNT).await.unwrap();
        let rejected = vault
            .create_fixture_candidate_session(SITE, ACCOUNT, "rejected")
            .unwrap();
        assert_eq!(
            attempt.reject_candidate(&rejected, ready_observation()),
            Err(BrowserAuthRuntimeError::CandidateRejected)
        );
        assert_eq!(
            vault.active_session(SITE, ACCOUNT).unwrap(),
            Some(old.clone())
        );

        let accepted = vault
            .create_fixture_candidate_session(SITE, ACCOUNT, "accepted")
            .unwrap();
        let handoff = attempt
            .accept_candidate(accepted.clone(), ready_observation())
            .unwrap();
        assert_eq!(handoff.site_id(), SITE);
        assert_eq!(handoff.account_ref(), ACCOUNT);
        assert_eq!(vault.active_session(SITE, ACCOUNT).unwrap(), Some(accepted));
        assert!(!format!("{handoff:?}").contains("accepted"));
        attempt.cleanup();
    }

    #[tokio::test]
    async fn expiry_and_crash_cleanup_do_not_touch_vault_active_session() {
        let vault = SessionVault::isolated_test();
        vault.register_account(SITE, ACCOUNT, "fixture").unwrap();
        let old = vault
            .create_fixture_candidate_session(SITE, ACCOUNT, "stable")
            .unwrap();
        vault
            .validate_and_swap(&old, CandidateValidation::Valid)
            .unwrap();
        let runtime =
            BrowserAuthRuntime::with_ttl(FakeBrowserWorker::new(), vault.clone(), Duration::ZERO);
        let mut attempt = runtime.start(SITE, ACCOUNT).await.unwrap();
        assert_eq!(
            attempt.request_input(BrowserInput::Submit).await,
            Err(BrowserAuthRuntimeError::Browser(
                BrowserError::SessionExpired
            ))
        );
        assert_eq!(vault.active_session(SITE, ACCOUNT).unwrap(), Some(old));
        assert!(
            attempt
                .events_after(0)
                .iter()
                .any(|event| { event.observation.state == BrowserAuthState::Expired })
        );
    }

    #[tokio::test]
    async fn candidate_capture_is_bound_one_shot_and_atomically_swapped() {
        let vault = SessionVault::isolated_test();
        vault.register_account(SITE, ACCOUNT, "fixture").unwrap();
        let old = vault
            .create_fixture_candidate_session(SITE, ACCOUNT, "old")
            .unwrap();
        vault
            .validate_and_swap(&old, CandidateValidation::Valid)
            .unwrap();
        let worker = FakeBrowserWorker::new();
        let runtime = BrowserAuthRuntime::new(worker.clone(), vault.clone());
        let mut attempt = runtime.start(SITE, ACCOUNT).await.unwrap();
        worker
            .set_candidate_material(
                attempt.session().id(),
                BrowserCandidateMaterial::fixture("captured-secret"),
            )
            .unwrap();
        let observation = ready_observation();
        let candidate = attempt
            .capture_candidate("capture-1", observation)
            .await
            .unwrap();
        assert!(vault.has_session(&candidate));
        assert_eq!(
            attempt
                .capture_candidate("capture-1", observation)
                .await
                .unwrap(),
            candidate
        );
        assert_eq!(
            attempt.capture_candidate("capture-2", observation).await,
            Err(BrowserAuthRuntimeError::CandidateCaptureRequestMismatch)
        );
        let handoff = attempt
            .accept_candidate_ref(candidate.session_id(), observation)
            .unwrap();
        assert_eq!(handoff.account_ref(), ACCOUNT);
        assert_eq!(
            vault.active_session(SITE, ACCOUNT).unwrap(),
            Some(candidate)
        );
        assert!(!format!("{handoff:?}").contains("captured-secret"));
    }

    #[tokio::test]
    async fn cancelled_capture_removes_only_new_candidate_and_preserves_old_session() {
        let vault = SessionVault::isolated_test();
        vault.register_account(SITE, ACCOUNT, "fixture").unwrap();
        let old = vault
            .create_fixture_candidate_session(SITE, ACCOUNT, "old")
            .unwrap();
        vault
            .validate_and_swap(&old, CandidateValidation::Valid)
            .unwrap();
        let worker = FakeBrowserWorker::new();
        let runtime = BrowserAuthRuntime::new(worker.clone(), vault.clone());
        let mut attempt = runtime.start(SITE, ACCOUNT).await.unwrap();
        worker
            .set_candidate_material(
                attempt.session().id(),
                BrowserCandidateMaterial::fixture("new"),
            )
            .unwrap();
        let candidate = attempt
            .capture_candidate("capture-cancel", ready_observation())
            .await
            .unwrap();
        assert!(vault.has_session(&candidate));
        attempt.cancel().unwrap();
        assert!(!vault.has_session(&candidate));
        assert_eq!(vault.active_session(SITE, ACCOUNT).unwrap(), Some(old));
    }

    #[tokio::test]
    async fn rejected_capture_is_not_replayed_by_duplicate_request() {
        let vault = SessionVault::isolated_test();
        vault.register_account(SITE, ACCOUNT, "fixture").unwrap();
        let old = vault
            .create_fixture_candidate_session(SITE, ACCOUNT, "old")
            .unwrap();
        vault
            .validate_and_swap(&old, CandidateValidation::Valid)
            .unwrap();
        let worker = FakeBrowserWorker::new();
        let runtime = BrowserAuthRuntime::new(worker.clone(), vault.clone());
        let mut attempt = runtime.start(SITE, ACCOUNT).await.unwrap();
        worker
            .set_candidate_material(
                attempt.session().id(),
                BrowserCandidateMaterial::fixture("reject"),
            )
            .unwrap();
        let observation = ready_observation();
        let candidate = attempt
            .capture_candidate("capture-reject", observation)
            .await
            .unwrap();
        let stale = BrowserAuthObservation {
            sequence: observation.sequence + 1,
            ..observation
        };
        assert_eq!(
            attempt.accept_candidate_ref(candidate.session_id(), stale),
            Err(BrowserAuthRuntimeError::InvalidCandidate)
        );
        assert!(!vault.has_session(&candidate));
        assert_eq!(vault.active_session(SITE, ACCOUNT).unwrap(), Some(old));
        assert_eq!(
            attempt
                .capture_candidate("capture-reject", observation)
                .await,
            Err(BrowserAuthRuntimeError::Browser(
                BrowserError::CandidateCaptureUnavailable
            ))
        );
    }
}
