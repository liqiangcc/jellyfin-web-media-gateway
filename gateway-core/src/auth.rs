//! Generic source-site authentication infrastructure.
//!
//! The vault in this module is deliberately an in-memory implementation.  It
//! is the executable test/storage boundary for the MVP contract; production
//! encryption and filesystem deployment are separate concerns.  Callers that
//! are not trusted server infrastructure only receive references or scoped
//! capabilities.  In particular, `SiteAccessCapability` never contains the
//! `SecretMaterial` stored here.

use crate::browser::ProfileAttachmentRef;
use crate::security::{
    EgressPolicy, EgressPolicyError, EgressScope, SiteAccessCapability, SiteAccessError,
    is_secret_header,
};
use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, LOCATION, SET_COOKIE};
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use site_adapter_api::SourceLocator;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AccountState {
    Unknown,
    Checking,
    Valid,
    Expired,
    LoginRequired,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SiteSessionRef {
    site_id: String,
    account_ref: String,
    session_id: String,
}

impl SiteSessionRef {
    fn new(site_id: &str, account_ref: &str) -> Self {
        Self {
            site_id: site_id.to_string(),
            account_ref: account_ref.to_string(),
            session_id: Uuid::new_v4().simple().to_string(),
        }
    }

    pub fn site_id(&self) -> &str {
        &self.site_id
    }

    pub fn account_ref(&self) -> &str {
        &self.account_ref
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SiteAccount {
    site_id: String,
    account_ref: String,
    label: String,
    state: AccountState,
    active_session: Option<SiteSessionRef>,
}

impl SiteAccount {
    pub fn site_id(&self) -> &str {
        &self.site_id
    }

    pub fn account_ref(&self) -> &str {
        &self.account_ref
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn state(&self) -> AccountState {
        self.state
    }

    pub fn active_session(&self) -> Option<&SiteSessionRef> {
        self.active_session.as_ref()
    }
}

/// Secret material is only constructed and consumed by trusted server-side
/// infrastructure.  Its Debug representation intentionally never includes
/// field values.
#[derive(Clone, Eq, PartialEq)]
struct SecretMaterial {
    cookie_header: Option<String>,
    authorization_header: Option<String>,
    local_storage: Option<String>,
    browser_profile: Option<Vec<u8>>,
}

impl SecretMaterial {
    fn new(
        cookie_header: Option<String>,
        authorization_header: Option<String>,
        local_storage: Option<String>,
        browser_profile: Option<Vec<u8>>,
    ) -> Self {
        Self {
            cookie_header,
            authorization_header,
            local_storage,
            browser_profile,
        }
    }

    fn is_empty(&self) -> bool {
        self.cookie_header.is_none()
            && self.authorization_header.is_none()
            && self.local_storage.is_none()
            && self.browser_profile.is_none()
    }

    fn cookie_header(&self) -> Option<&str> {
        self.cookie_header.as_deref()
    }

    fn authorization_header(&self) -> Option<&str> {
        self.authorization_header.as_deref()
    }
}

impl fmt::Debug for SecretMaterial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretMaterial")
            .field(
                "cookie_header",
                &self.cookie_header.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "authorization_header",
                &self.authorization_header.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "local_storage",
                &self.local_storage.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "browser_profile",
                &self.browser_profile.as_ref().map(|profile| profile.len()),
            )
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultError {
    InvalidReference,
    AccountNotFound,
    SessionNotFound,
    CandidateNotFound,
    CandidateRejected,
    CandidateCancelled,
    SessionNotActive,
    EmptySecretMaterial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateValidation {
    Valid,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CleanupResult {
    NoPreviousSession,
    PreviousSessionRemoved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSwapResult {
    pub active_session: SiteSessionRef,
    pub replaced_session: Option<SiteSessionRef>,
    pub cleanup: CleanupResult,
}

#[derive(Clone)]
pub struct SessionVault {
    inner: Arc<Mutex<VaultInner>>,
}

#[derive(Default)]
struct VaultInner {
    accounts: HashMap<(String, String), SiteAccount>,
    sessions: HashMap<String, StoredSession>,
    active_account_by_site: HashMap<String, String>,
}

struct StoredSession {
    reference: SiteSessionRef,
    material: SecretMaterial,
    candidate: bool,
}

impl Default for SessionVault {
    fn default() -> Self {
        Self::isolated_test()
    }
}

impl SessionVault {
    /// Returns a fresh isolated vault suitable for deterministic tests and
    /// local contract execution.  It has no relation to a production path.
    pub fn isolated_test() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VaultInner::default())),
        }
    }

    pub fn register_account(
        &self,
        site_id: impl Into<String>,
        account_ref: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<SiteAccount, VaultError> {
        let site_id = site_id.into();
        let account_ref = account_ref.into();
        if site_id.is_empty() || account_ref.is_empty() {
            return Err(VaultError::InvalidReference);
        }
        let mut inner = self.inner.lock().expect("session vault poisoned");
        let key = (site_id.clone(), account_ref.clone());
        if inner.accounts.contains_key(&key) {
            return Err(VaultError::InvalidReference);
        }
        let account = SiteAccount {
            site_id,
            account_ref,
            label: label.into(),
            state: AccountState::Unknown,
            active_session: None,
        };
        inner.accounts.insert(key, account.clone());
        Ok(account)
    }

    pub fn account(&self, site_id: &str, account_ref: &str) -> Result<SiteAccount, VaultError> {
        self.inner
            .lock()
            .expect("session vault poisoned")
            .accounts
            .get(&(site_id.to_string(), account_ref.to_string()))
            .cloned()
            .ok_or(VaultError::AccountNotFound)
    }

    pub fn set_account_state(
        &self,
        site_id: &str,
        account_ref: &str,
        state: AccountState,
    ) -> Result<SiteAccount, VaultError> {
        let mut inner = self.inner.lock().expect("session vault poisoned");
        let account = inner
            .accounts
            .get_mut(&(site_id.to_string(), account_ref.to_string()))
            .ok_or(VaultError::AccountNotFound)?;
        account.state = state;
        Ok(account.clone())
    }

    fn create_candidate_session(
        &self,
        site_id: &str,
        account_ref: &str,
        material: SecretMaterial,
    ) -> Result<SiteSessionRef, VaultError> {
        if material.is_empty() {
            return Err(VaultError::EmptySecretMaterial);
        }
        let mut inner = self.inner.lock().expect("session vault poisoned");
        if !inner
            .accounts
            .contains_key(&(site_id.to_string(), account_ref.to_string()))
        {
            return Err(VaultError::AccountNotFound);
        }
        let reference = SiteSessionRef::new(site_id, account_ref);
        inner.sessions.insert(
            reference.session_id.clone(),
            StoredSession {
                reference: reference.clone(),
                material,
                candidate: true,
            },
        );
        Ok(reference)
    }

    /// Creates a deterministic fake session for contract tests without
    /// exposing a public raw-secret insertion or retrieval API.  Production
    /// auth flows use the private server-side candidate method above.
    pub fn create_fixture_candidate_session(
        &self,
        site_id: &str,
        account_ref: &str,
        fixture_label: &str,
    ) -> Result<SiteSessionRef, VaultError> {
        self.create_candidate_session(
            site_id,
            account_ref,
            SecretMaterial::new(
                Some(format!("session-cookie-{fixture_label}")),
                Some(format!("Bearer session-token-{fixture_label}")),
                Some(format!("local-storage-{fixture_label}")),
                Some(format!("profile-{fixture_label}").into_bytes()),
            ),
        )
    }

    pub fn validate_and_swap(
        &self,
        candidate: &SiteSessionRef,
        validation: CandidateValidation,
    ) -> Result<SessionSwapResult, VaultError> {
        let mut inner = self.inner.lock().expect("session vault poisoned");
        let Some(stored) = inner.sessions.get(candidate.session_id()) else {
            return Err(VaultError::CandidateNotFound);
        };
        if !stored.candidate || stored.reference != *candidate {
            return Err(VaultError::CandidateNotFound);
        }
        if validation == CandidateValidation::Invalid {
            inner.sessions.remove(candidate.session_id());
            return Err(VaultError::CandidateRejected);
        }

        let key = (candidate.site_id.clone(), candidate.account_ref.clone());
        let previous_site_account = inner
            .active_account_by_site
            .get(candidate.site_id())
            .cloned();
        let mut replaced_from_other_account = None;
        if let Some(previous_account_ref) = previous_site_account
            .as_deref()
            .filter(|previous| *previous != candidate.account_ref())
        {
            if let Some(previous_account) = inner
                .accounts
                .get_mut(&(candidate.site_id.clone(), previous_account_ref.to_string()))
            {
                replaced_from_other_account = previous_account.active_session.take();
                previous_account.state = AccountState::LoginRequired;
            }
            if let Some(previous) = replaced_from_other_account.as_ref() {
                inner.sessions.remove(previous.session_id());
            }
        }
        let account = inner
            .accounts
            .get_mut(&key)
            .ok_or(VaultError::AccountNotFound)?;
        let previous = account.active_session.replace(candidate.clone());
        account.state = AccountState::Valid;
        inner
            .active_account_by_site
            .insert(candidate.site_id.clone(), candidate.account_ref.clone());
        if let Some(previous) = previous.as_ref() {
            inner.sessions.remove(previous.session_id());
        }
        if let Some(stored) = inner.sessions.get_mut(candidate.session_id()) {
            stored.candidate = false;
        }
        let replaced_session = previous.or(replaced_from_other_account);
        Ok(SessionSwapResult {
            active_session: candidate.clone(),
            cleanup: if replaced_session.is_some() {
                CleanupResult::PreviousSessionRemoved
            } else {
                CleanupResult::NoPreviousSession
            },
            replaced_session,
        })
    }

    pub fn cancel_candidate(&self, candidate: &SiteSessionRef) -> Result<(), VaultError> {
        let mut inner = self.inner.lock().expect("session vault poisoned");
        let Some(stored) = inner.sessions.get(candidate.session_id()) else {
            return Err(VaultError::CandidateNotFound);
        };
        if !stored.candidate || stored.reference != *candidate {
            return Err(VaultError::CandidateNotFound);
        }
        inner.sessions.remove(candidate.session_id());
        Ok(())
    }

    pub fn active_session(
        &self,
        site_id: &str,
        account_ref: &str,
    ) -> Result<Option<SiteSessionRef>, VaultError> {
        Ok(self.account(site_id, account_ref)?.active_session)
    }

    pub fn mark_expired(
        &self,
        site_id: &str,
        account_ref: &str,
    ) -> Result<SiteAccount, VaultError> {
        self.set_account_state(site_id, account_ref, AccountState::Expired)
    }

    pub fn logout(&self, site_id: &str, account_ref: &str) -> Result<SiteAccount, VaultError> {
        let mut inner = self.inner.lock().expect("session vault poisoned");
        let key = (site_id.to_string(), account_ref.to_string());
        let active = inner
            .accounts
            .get_mut(&key)
            .ok_or(VaultError::AccountNotFound)?
            .active_session
            .take();
        if let Some(active) = active {
            inner.sessions.remove(active.session_id());
        }
        let account_was_active = inner
            .active_account_by_site
            .get(site_id)
            .map(String::as_str)
            == Some(account_ref);
        let account = inner
            .accounts
            .get_mut(&key)
            .ok_or(VaultError::AccountNotFound)?;
        account.state = AccountState::LoginRequired;
        let result = account.clone();
        if account_was_active {
            inner.active_account_by_site.remove(site_id);
        }
        Ok(result)
    }

    fn read_secret(&self, session: &SiteSessionRef) -> Result<SecretMaterial, VaultError> {
        self.inner
            .lock()
            .expect("session vault poisoned")
            .sessions
            .get(session.session_id())
            .filter(|stored| !stored.candidate && stored.reference == *session)
            .map(|stored| stored.material.clone())
            .ok_or(VaultError::SessionNotActive)
    }

    pub fn has_session(&self, session: &SiteSessionRef) -> bool {
        self.inner
            .lock()
            .expect("session vault poisoned")
            .sessions
            .get(session.session_id())
            .is_some_and(|stored| stored.reference == *session)
    }

    /// Issue an opaque browser-profile attachment capability from the
    /// Core/Vault boundary.  Browser workers consume this ref but cannot issue
    /// or materialize profile material themselves.
    #[allow(dead_code)]
    pub(crate) fn issue_profile_attachment_ref(
        &self,
        session: &SiteSessionRef,
    ) -> Result<ProfileAttachmentRef, VaultError> {
        let inner = self.inner.lock().expect("session vault poisoned");
        if inner
            .sessions
            .get(session.session_id())
            .filter(|stored| !stored.candidate && stored.reference == *session)
            .is_none()
        {
            return Err(VaultError::SessionNotActive);
        }
        Ok(ProfileAttachmentRef::from_vault_issued(
            Uuid::new_v4().simple().to_string(),
        ))
    }

    pub fn authorize_capability(
        &self,
        capability: &SiteAccessCapability,
        site_id: &str,
        account_ref: Option<&str>,
        url: &Url,
    ) -> Result<SiteSessionRef, AuthBoundaryError> {
        capability
            .authorize_for_account(site_id, account_ref, url)
            .map_err(AuthBoundaryError::Capability)?;
        let session = capability
            .session_id()
            .ok_or(AuthBoundaryError::Capability(
                SiteAccessError::SessionNotBound,
            ))?;
        let inner = self.inner.lock().expect("session vault poisoned");
        let Some(stored) = inner
            .sessions
            .get(session)
            .filter(|stored| !stored.candidate)
        else {
            // A session-bound capability whose material disappeared is a
            // revoked/stale capability, never a reason to expose vault state.
            return Err(AuthBoundaryError::Capability(SiteAccessError::StaleSession));
        };
        let account = inner
            .accounts
            .get(&(
                stored.reference.site_id.clone(),
                stored.reference.account_ref.clone(),
            ))
            .ok_or(AuthBoundaryError::Vault(VaultError::AccountNotFound))?;
        if account.state != AccountState::Valid {
            return Err(AuthBoundaryError::Capability(
                SiteAccessError::SessionExpired,
            ));
        }
        if account.active_session.as_ref() != Some(&stored.reference)
            || inner
                .active_account_by_site
                .get(site_id)
                .map(String::as_str)
                != Some(stored.reference.account_ref())
        {
            return Err(AuthBoundaryError::Capability(SiteAccessError::StaleSession));
        }
        Ok(stored.reference.clone())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SiteAccessContext {
    site_id: String,
    account_ref: Option<String>,
    context_id: String,
}

impl SiteAccessContext {
    /// Issues an independent server-side request context.  It is intentionally
    /// separate from the presented capability so a caller cannot self-attest
    /// the expected site by copying fields from that capability.
    pub fn issue(site_id: impl Into<String>, account_ref: Option<String>) -> Self {
        Self {
            site_id: site_id.into(),
            account_ref,
            context_id: Uuid::new_v4().simple().to_string(),
        }
    }

    pub fn site_id(&self) -> &str {
        &self.site_id
    }

    pub fn account_ref(&self) -> Option<&str> {
        self.account_ref.as_deref()
    }

    pub fn context_id(&self) -> &str {
        &self.context_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingSourceLocator {
    pub site_id: String,
    pub plugin_id: String,
    pub locator_version: u32,
    pub opaque_payload: String,
}

impl From<&SourceLocator> for PendingSourceLocator {
    fn from(locator: &SourceLocator) -> Self {
        Self {
            site_id: locator.site_id.clone(),
            plugin_id: locator.plugin_id.clone(),
            locator_version: locator.locator_version,
            opaque_payload: locator.opaque_payload.clone(),
        }
    }
}

impl From<PendingSourceLocator> for SourceLocator {
    fn from(locator: PendingSourceLocator) -> Self {
        Self {
            site_id: locator.site_id,
            plugin_id: locator.plugin_id,
            locator_version: locator.locator_version,
            opaque_payload: locator.opaque_payload,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PendingPlaybackAction {
    Play,
    Resume,
    Handoff,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingIntent {
    pub intent_id: String,
    pub site_id: String,
    pub source_locator: PendingSourceLocator,
    pub display_ref: Option<String>,
    pub action: PendingPlaybackAction,
    pub expected_session_revision: Option<u64>,
    pub created_at_ms: u64,
}

impl PendingIntent {
    pub fn new(
        intent_id: impl Into<String>,
        locator: &SourceLocator,
        display_ref: Option<String>,
        action: PendingPlaybackAction,
        expected_session_revision: Option<u64>,
    ) -> Self {
        Self {
            intent_id: intent_id.into(),
            site_id: locator.site_id.clone(),
            source_locator: locator.into(),
            display_ref,
            action,
            expected_session_revision,
            created_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    pub fn to_source_locator(&self) -> SourceLocator {
        self.source_locator.clone().into()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum AuthBoundaryError {
    Capability(SiteAccessError),
    Vault(VaultError),
    Egress(EgressPolicyError),
    MissingLocation,
    InvalidRedirect,
    TooManyRedirects,
    Transport,
    ResponseBody,
}

impl fmt::Display for AuthBoundaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Capability(_) => "capability rejected",
            Self::Vault(_) => "vault access rejected",
            Self::Egress(_) => "egress rejected",
            Self::MissingLocation => "redirect missing location",
            Self::InvalidRedirect => "redirect rejected",
            Self::TooManyRedirects => "too many redirects",
            Self::Transport => "upstream transport failed",
            Self::ResponseBody => "upstream response body failed",
        };
        formatter.write_str(label)
    }
}

impl std::error::Error for AuthBoundaryError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedHttpResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

/// A server-owned HTTP client.  Its caller supplies a method and URL only;
/// authentication headers are obtained from the Vault after capability and
/// central EgressPolicy validation.  Redirects repeat both validations.
#[derive(Clone)]
pub struct ScopedSiteHttpClient {
    vault: SessionVault,
    egress: Arc<EgressPolicy>,
    scope: EgressScope,
    context: SiteAccessContext,
}

impl ScopedSiteHttpClient {
    pub fn new(
        vault: SessionVault,
        egress: Arc<EgressPolicy>,
        scope: EgressScope,
        context: SiteAccessContext,
    ) -> Self {
        Self {
            vault,
            egress,
            scope,
            context,
        }
    }

    pub async fn request(
        &self,
        capability: &SiteAccessCapability,
        method: Method,
        url: Url,
    ) -> Result<ScopedHttpResponse, AuthBoundaryError> {
        let mut current = url;
        for redirect_count in 0..=5 {
            let session = self.vault.authorize_capability(
                capability,
                self.context.site_id(),
                self.context.account_ref(),
                &current,
            )?;
            let target = self
                .egress
                .validate_and_resolve(&current, &self.scope)
                .await
                .map_err(AuthBoundaryError::Egress)?;
            let secret = self
                .vault
                .read_secret(&session)
                .map_err(AuthBoundaryError::Vault)?;
            let client = target
                .pinned_client_with_timeout(Some(Duration::from_secs(15)))
                .map_err(|_| AuthBoundaryError::Transport)?;
            let mut request = client.request(method.clone(), current.clone());
            if let Some(cookie) = secret.cookie_header() {
                request = request.header("cookie", cookie);
            }
            if let Some(authorization) = secret.authorization_header() {
                request = request.header("authorization", authorization);
            }
            let response = request
                .send()
                .await
                .map_err(|_| AuthBoundaryError::Transport)?;
            if response.status().is_redirection() {
                if redirect_count == 5 {
                    return Err(AuthBoundaryError::TooManyRedirects);
                }
                let location = response
                    .headers()
                    .get(LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or(AuthBoundaryError::MissingLocation)?;
                current = current
                    .join(location)
                    .map_err(|_| AuthBoundaryError::InvalidRedirect)?;
                continue;
            }
            let status = response.status();
            let headers = safe_response_headers(response.headers());
            let body = response
                .bytes()
                .await
                .map_err(|_| AuthBoundaryError::ResponseBody)?;
            return Ok(ScopedHttpResponse {
                status,
                headers,
                body,
            });
        }
        Err(AuthBoundaryError::TooManyRedirects)
    }
}

fn safe_response_headers(headers: &HeaderMap) -> HeaderMap {
    let mut safe = HeaderMap::new();
    for (name, value) in headers {
        if name == SET_COOKIE || is_secret_header(name.as_str(), value.to_str().unwrap_or_default())
        {
            continue;
        }
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            safe.append(name, value);
        }
    }
    safe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_material_debug_is_redacted_inside_the_vault_boundary() {
        let material = SecretMaterial::new(
            Some("cookie-secret-sentinel".into()),
            Some("Bearer authorization-secret-sentinel".into()),
            Some("local-storage-secret-sentinel".into()),
            Some(b"profile-secret-sentinel".to_vec()),
        );
        let diagnostic = format!("{material:?}");
        assert!(!diagnostic.contains("secret-sentinel"));
        assert!(diagnostic.contains("[REDACTED]"));
    }

    #[test]
    fn active_vault_session_issues_opaque_profile_attachment_ref() {
        let vault = SessionVault::isolated_test();
        vault
            .register_account("site-a", "account-a", "fixture")
            .unwrap();
        let candidate = vault
            .create_fixture_candidate_session("site-a", "account-a", "profile")
            .unwrap();
        vault
            .validate_and_swap(&candidate, CandidateValidation::Valid)
            .unwrap();
        let profile = vault.issue_profile_attachment_ref(&candidate).unwrap();
        let diagnostic = format!("{profile:?}");
        assert!(!diagnostic.contains("profile"));
    }
}
