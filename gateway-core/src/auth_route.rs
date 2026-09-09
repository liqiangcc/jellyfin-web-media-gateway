//! Gateway-owned HTTP coordinator for authenticated Browser Worker attempts.
//!
//! The route layer owns only opaque attempt identity, request idempotency and
//! lifecycle.  Browser facts remain generic and the SiteAdapter/SourceSession
//! contracts remain the only place that interprets a source.

use crate::auth::{SessionVault, VaultError};
#[cfg(test)]
use crate::browser::FakeBrowserWorker;
use crate::browser::{
    BrowserEvent, BrowserEventKind, BrowserInput, BrowserNavigationRequest,
    BrowserObservationHandoff, BrowserOperationId, PointerButton, R008NavigationPolicy,
};
use crate::browser_auth::{BrowserAuthAttempt, BrowserAuthRuntime, BrowserAuthRuntimeError};
use crate::browser_chromium::ChromiumBrowserWorker;
use crate::source_session::CreateSessionRequest;
use crate::{GatewayService, GatewayState};
use axum::Json;
use axum::extract::{Path, Query, State, rejection::JsonRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use site_adapter_api::{
    AuthenticatedSessionHandoff, BROWSER_AUTH_OBSERVATION_VERSION, BrowserAuthDiagnostic,
    BrowserAuthObservation, BrowserAuthState, SourceLocator,
};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Mutex as AsyncMutex;
use url::Url;
use uuid::Uuid;

const MAX_REQUEST_ID: usize = 128;
const MAX_SITE_ID: usize = 128;
const MAX_ACCOUNT_REF: usize = 256;
const MAX_SOURCE: usize = 4096;
const MAX_CANDIDATE_REF: usize = 256;
const MAX_INPUT: usize = 2048;
const MAX_EVENTS: usize = 64;
const MAX_CURSOR: u64 = 1_000_000_000;
const HANDOFF_TTL: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub(crate) struct AuthRouteCoordinator {
    runtime: AuthRuntime,
    vault: SessionVault,
    attempts: Arc<Mutex<HashMap<String, AttemptRecord>>>,
    start_requests: Arc<Mutex<HashMap<String, StartRecord>>>,
    cancelled_requests: Arc<Mutex<HashMap<(String, String), ()>>>,
    start_gate: Arc<AsyncMutex<()>>,
    operation_gate: Arc<AsyncMutex<()>>,
}

#[derive(Clone)]
enum AuthRuntime {
    Chromium(BrowserAuthRuntime<ChromiumBrowserWorker>),
    #[cfg(test)]
    Fake(BrowserAuthRuntime<FakeBrowserWorker>),
}

enum AuthAttempt {
    Chromium(BrowserAuthAttempt<ChromiumBrowserWorker>),
    #[cfg(test)]
    Fake(BrowserAuthAttempt<FakeBrowserWorker>),
}

struct AttemptRecord {
    attempt: AuthAttempt,
    requests: HashMap<String, u64>,
    last_operation: u64,
    accepted: Option<AcceptedHandoff>,
    playback_requests: HashMap<String, u64>,
}

#[derive(Clone)]
struct AcceptedHandoff {
    browser: BrowserObservationHandoff,
    authenticated: AuthenticatedSessionHandoff,
}

#[derive(Clone)]
struct StartRecord {
    fingerprint: u64,
    attempt_id: String,
}

impl AuthRouteCoordinator {
    pub(crate) fn new(vault: SessionVault) -> Self {
        let worker = ChromiumBrowserWorker::new();
        let runtime = BrowserAuthRuntime::new(worker, vault.clone());
        Self {
            runtime: AuthRuntime::Chromium(runtime),
            vault,
            attempts: Arc::new(Mutex::new(HashMap::new())),
            start_requests: Arc::new(Mutex::new(HashMap::new())),
            cancelled_requests: Arc::new(Mutex::new(HashMap::new())),
            start_gate: Arc::new(AsyncMutex::new(())),
            operation_gate: Arc::new(AsyncMutex::new(())),
        }
    }

    #[cfg(test)]
    pub(crate) fn fake_for_tests(vault: SessionVault) -> Self {
        Self {
            runtime: AuthRuntime::Fake(BrowserAuthRuntime::new(
                FakeBrowserWorker::new(),
                vault.clone(),
            )),
            vault,
            attempts: Arc::new(Mutex::new(HashMap::new())),
            start_requests: Arc::new(Mutex::new(HashMap::new())),
            cancelled_requests: Arc::new(Mutex::new(HashMap::new())),
            start_gate: Arc::new(AsyncMutex::new(())),
            operation_gate: Arc::new(AsyncMutex::new(())),
        }
    }

    pub(crate) fn register_account(
        &self,
        site_id: impl Into<String>,
        account_ref: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<(), VaultError> {
        self.vault
            .register_account(site_id, account_ref, label)
            .map(|_| ())
    }

    async fn start(
        &self,
        request_id: &str,
        site_id: &str,
        account_ref: &str,
    ) -> Result<(String, BrowserAuthState, u64, bool), ApiError> {
        let _start_guard = self.start_gate.lock().await;
        let fingerprint = fingerprint(&(site_id, account_ref));
        if let Some(existing) = self
            .start_requests
            .lock()
            .expect("auth start store poisoned")
            .get(request_id)
            .cloned()
        {
            if existing.fingerprint != fingerprint {
                return Err(ApiError::conflict("AUTH_REQUEST_ID_MISMATCH"));
            }
            let expired = {
                let attempts = self.attempts.lock().expect("auth attempt store poisoned");
                attempts
                    .get(&existing.attempt_id)
                    .is_none_or(AttemptRecord::is_expired)
            };
            if expired {
                self.expire_and_remove(&existing.attempt_id);
                self.start_requests
                    .lock()
                    .expect("auth start store poisoned")
                    .remove(request_id);
                return Err(ApiError::conflict("AUTH_ATTEMPT_EXPIRED"));
            }
            let attempts = self.attempts.lock().expect("auth attempt store poisoned");
            let record = attempts
                .get(&existing.attempt_id)
                .expect("non-expired auth attempt disappeared");
            return Ok((
                existing.attempt_id,
                record.state(),
                record.expires_in(),
                true,
            ));
        }

        let attempt = match &self.runtime {
            AuthRuntime::Chromium(runtime) => runtime
                .start(site_id.to_owned(), account_ref.to_owned())
                .await
                .map(AuthAttempt::Chromium),
            #[cfg(test)]
            AuthRuntime::Fake(runtime) => runtime
                .start(site_id.to_owned(), account_ref.to_owned())
                .await
                .map(AuthAttempt::Fake),
        }
        .map_err(ApiError::from_runtime)?;
        let attempt_id = format!("a-{}", Uuid::new_v4().simple());
        let record = AttemptRecord {
            attempt,
            requests: HashMap::new(),
            last_operation: 0,
            accepted: None,
            playback_requests: HashMap::new(),
        };
        let state = record.state();
        let expires_in = record.expires_in();
        self.attempts
            .lock()
            .expect("auth attempt store poisoned")
            .insert(attempt_id.clone(), record);
        self.start_requests
            .lock()
            .expect("auth start store poisoned")
            .insert(
                request_id.to_owned(),
                StartRecord {
                    fingerprint,
                    attempt_id: attempt_id.clone(),
                },
            );
        Ok((attempt_id, state, expires_in, false))
    }

    fn take(&self, id: &str) -> Result<AttemptRecord, ApiError> {
        let mut attempts = self.attempts.lock().expect("auth attempt store poisoned");
        let mut record = attempts
            .remove(id)
            .ok_or_else(|| ApiError::not_found("AUTH_ATTEMPT_NOT_FOUND"))?;
        if record.is_expired() {
            record.expire();
            self.remove_start_for_attempt(id);
            return Err(ApiError::conflict("AUTH_ATTEMPT_EXPIRED"));
        }
        Ok(record)
    }

    fn expire_and_remove(&self, id: &str) {
        if let Some(mut record) = self
            .attempts
            .lock()
            .expect("auth attempt store poisoned")
            .remove(id)
        {
            record.expire();
        }
        self.remove_start_for_attempt(id);
    }

    fn remove_start_for_attempt(&self, id: &str) {
        self.start_requests
            .lock()
            .expect("auth start store poisoned")
            .retain(|_, record| record.attempt_id != id);
    }

    fn put(&self, id: String, record: AttemptRecord) {
        self.attempts
            .lock()
            .expect("auth attempt store poisoned")
            .insert(id, record);
    }

    async fn navigate(
        &self,
        id: &str,
        request_id: &str,
        operation_id: u64,
        url: Url,
        egress: crate::EgressPolicy,
    ) -> Result<(u64, bool), ApiError> {
        let _operation_guard = self.operation_gate.lock().await;
        let mut record = self.take(id)?;
        let request_fingerprint = fingerprint(&(operation_id, url.as_str()));
        if let Some(previous) = record.requests.get(request_id).copied() {
            if previous != request_fingerprint {
                self.put(id.to_owned(), record);
                return Err(ApiError::conflict("AUTH_REQUEST_ID_MISMATCH"));
            }
            self.put(id.to_owned(), record);
            return Ok((operation_id, true));
        }
        if operation_id == 0 || operation_id <= record.last_operation {
            self.put(id.to_owned(), record);
            return Err(ApiError::conflict("AUTH_OPERATION_STALE"));
        }
        let request = BrowserNavigationRequest::with_operation_id(
            BrowserOperationId::from_value(operation_id),
            url,
        );
        let result = record
            .navigate(request, &R008NavigationPolicy::public_web(egress))
            .await;
        match result {
            Ok(operation) => {
                record.last_operation = operation.value();
                record
                    .requests
                    .insert(request_id.to_owned(), request_fingerprint);
                self.put(id.to_owned(), record);
                Ok((operation.value(), false))
            }
            Err(error) => {
                self.put(id.to_owned(), record);
                Err(ApiError::from_runtime(error))
            }
        }
    }

    async fn input(
        &self,
        id: &str,
        request_id: &str,
        input: BrowserInput,
    ) -> Result<bool, ApiError> {
        let _operation_guard = self.operation_gate.lock().await;
        let mut record = self.take(id)?;
        let input_fingerprint = fingerprint(&format!("{input:?}"));
        if let Some(previous) = record.requests.get(request_id).copied() {
            if previous != input_fingerprint {
                self.put(id.to_owned(), record);
                return Err(ApiError::conflict("AUTH_REQUEST_ID_MISMATCH"));
            }
            self.put(id.to_owned(), record);
            return Ok(true);
        }
        if let Err(error) = record.request_input(input).await {
            self.put(id.to_owned(), record);
            return Err(ApiError::from_runtime(error));
        }
        record
            .requests
            .insert(request_id.to_owned(), input_fingerprint);
        self.put(id.to_owned(), record);
        Ok(false)
    }

    fn events(
        &self,
        id: &str,
        after: u64,
    ) -> Result<(Vec<AuthEventView>, Vec<BrowserEventView>, AuthRouteStatus), ApiError> {
        if after > MAX_CURSOR {
            return Err(ApiError::bad_request("AUTH_CURSOR_INVALID"));
        }
        let mut record = self.take(id)?;
        let auth_events = record
            .auth_events_after(after)
            .into_iter()
            .take(MAX_EVENTS)
            .map(AuthEventView::from)
            .collect();
        let browser_events = match record.browser_events_after(after) {
            Ok(events) => events
                .into_iter()
                .take(MAX_EVENTS)
                .map(BrowserEventView::from)
                .collect(),
            Err(error) => {
                self.put(id.to_owned(), record);
                return Err(ApiError::from_runtime(error));
            }
        };
        let status = AuthRouteStatus::from(record.state());
        self.put(id.to_owned(), record);
        Ok((auth_events, browser_events, status))
    }

    fn cancel(&self, id: &str, request_id: &str) -> Result<(AuthRouteStatus, bool), ApiError> {
        if self
            .cancelled_requests
            .lock()
            .expect("auth cancellation store poisoned")
            .contains_key(&(id.to_owned(), request_id.to_owned()))
        {
            return Ok((AuthRouteStatus::Cancelled, true));
        }
        let mut record = self.take(id)?;
        if let Err(error) = record.cancel() {
            self.put(id.to_owned(), record);
            return Err(ApiError::from_runtime(error));
        }
        let status = AuthRouteStatus::from(record.state());
        // A cancelled attempt is removed after the worker has been closed.
        self.remove_start_for_attempt(id);
        self.cancelled_requests
            .lock()
            .expect("auth cancellation store poisoned")
            .insert((id.to_owned(), request_id.to_owned()), ());
        Ok((status, false))
    }

    fn candidate(
        &self,
        id: &str,
        request_id: &str,
        candidate: &CandidateRequest,
        locator: SourceLocator,
    ) -> Result<(bool, AuthRouteStatus), ApiError> {
        let mut record = self.take(id)?;
        let request_fingerprint = fingerprint(&(
            candidate.operation_id,
            candidate.candidate_session_id.as_str(),
            candidate.source.as_str(),
            candidate.observation.schema_version,
            candidate.observation.sequence,
        ));
        if let Some(previous) = record.requests.get(request_id).copied() {
            if previous != request_fingerprint {
                self.put(id.to_owned(), record);
                return Err(ApiError::conflict("AUTH_REQUEST_ID_MISMATCH"));
            }
            let status = AuthRouteStatus::from(record.state());
            self.put(id.to_owned(), record);
            return Ok((true, status));
        }
        if record.accepted.is_some() {
            self.put(id.to_owned(), record);
            return Err(ApiError::conflict("AUTH_CANDIDATE_ALREADY_ACCEPTED"));
        }
        let browser = match record.take_observation(locator, candidate.operation_id, HANDOFF_TTL) {
            Ok(browser) => browser,
            Err(error) => {
                self.put(id.to_owned(), record);
                return Err(ApiError::from_runtime(error));
            }
        };
        let observation = match candidate.observation.into_observation() {
            Ok(observation) => observation,
            Err(error) => {
                self.put(id.to_owned(), record);
                return Err(error);
            }
        };
        let authenticated =
            match record.accept_candidate_ref(&candidate.candidate_session_id, observation) {
                Ok(authenticated) => authenticated,
                Err(error) => {
                    self.put(id.to_owned(), record);
                    return Err(ApiError::from_runtime(error));
                }
            };
        record.accepted = Some(AcceptedHandoff {
            browser,
            authenticated,
        });
        record
            .requests
            .insert(request_id.to_owned(), request_fingerprint);
        let status = AuthRouteStatus::from(record.state());
        self.put(id.to_owned(), record);
        Ok((false, status))
    }

    fn playback(
        &self,
        gateway: &GatewayService,
        id: &str,
        request: CreateSessionRequest,
    ) -> Result<Response, ApiError> {
        let mut record = self.take(id)?;
        let Some(accepted) = record.accepted.clone() else {
            self.put(id.to_owned(), record);
            return Err(ApiError::conflict("AUTH_CANDIDATE_REQUIRED"));
        };
        let request_fingerprint =
            fingerprint(&(request.source.as_str(), request.display_id.as_str()));
        if let Some(previous) = record.playback_requests.get(&request.request_id) {
            if *previous != request_fingerprint {
                self.put(id.to_owned(), record);
                return Err(ApiError::conflict("AUTH_REQUEST_ID_MISMATCH"));
            }
        } else {
            record
                .playback_requests
                .insert(request.request_id.clone(), request_fingerprint);
        }
        let response = gateway.create_authenticated_playback_session(
            request,
            accepted.browser.clone(),
            accepted.authenticated.clone(),
        );
        self.put(id.to_owned(), record);
        Ok(response)
    }
}

impl AttemptRecord {
    fn state(&self) -> BrowserAuthState {
        match self {
            Self {
                attempt: AuthAttempt::Chromium(attempt),
                ..
            } => attempt.state(),
            #[cfg(test)]
            Self {
                attempt: AuthAttempt::Fake(attempt),
                ..
            } => attempt.state(),
        }
    }

    fn expires_in(&self) -> u64 {
        match &self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.expires_in().as_secs(),
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.expires_in().as_secs(),
        }
    }

    fn is_expired(&self) -> bool {
        match &self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.is_expired(),
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.is_expired(),
        }
    }

    fn expire(&mut self) {
        match &mut self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.expire(),
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.expire(),
        }
    }

    fn auth_events_after(&self, after: u64) -> Vec<crate::browser_auth::BrowserAuthEvent> {
        match &self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.events_after(after),
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.events_after(after),
        }
    }

    fn browser_events_after(
        &mut self,
        after: u64,
    ) -> Result<Vec<BrowserEvent>, BrowserAuthRuntimeError> {
        match &mut self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.browser_events_after(after),
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.browser_events_after(after),
        }
    }

    async fn navigate(
        &mut self,
        request: BrowserNavigationRequest,
        policy: &R008NavigationPolicy,
    ) -> Result<BrowserOperationId, BrowserAuthRuntimeError> {
        match &mut self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.navigate(request, policy).await,
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.navigate(request, policy).await,
        }
    }

    async fn request_input(&mut self, input: BrowserInput) -> Result<(), BrowserAuthRuntimeError> {
        match &mut self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.request_input(input).await,
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.request_input(input).await,
        }
    }

    fn cancel(&mut self) -> Result<(), BrowserAuthRuntimeError> {
        match &mut self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.cancel(),
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.cancel(),
        }
    }

    fn take_observation(
        &mut self,
        locator: SourceLocator,
        operation_id: u64,
        ttl: Duration,
    ) -> Result<BrowserObservationHandoff, BrowserAuthRuntimeError> {
        match &mut self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.take_observation_handoff(
                locator,
                BrowserOperationId::from_value(operation_id),
                ttl,
            ),
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.take_observation_handoff(
                locator,
                BrowserOperationId::from_value(operation_id),
                ttl,
            ),
        }
    }

    fn accept_candidate_ref(
        &mut self,
        candidate: &str,
        observation: BrowserAuthObservation,
    ) -> Result<AuthenticatedSessionHandoff, BrowserAuthRuntimeError> {
        match &mut self.attempt {
            AuthAttempt::Chromium(attempt) => attempt.accept_candidate_ref(candidate, observation),
            #[cfg(test)]
            AuthAttempt::Fake(attempt) => attempt.accept_candidate_ref(candidate, observation),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StartRequest {
    pub request_id: String,
    pub site_id: String,
    pub account_ref: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NavigationRequest {
    pub request_id: String,
    pub operation_id: u64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InputRequest {
    pub request_id: String,
    #[serde(flatten)]
    pub input: InputDto,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum InputDto {
    Key { key: String },
    Pointer { x: i32, y: i32, button: String },
    Text { value: String },
    Submit,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CancelRequest {
    pub request_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CandidateRequest {
    pub request_id: String,
    pub source: String,
    pub operation_id: u64,
    pub candidate_session_id: String,
    pub observation: AuthObservationDto,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthObservationDto {
    pub schema_version: u32,
    pub sequence: u64,
    pub state: AuthStateDto,
    pub diagnostic: AuthDiagnosticDto,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthStateDto {
    Required,
    InputNeeded,
    CandidateReady,
    Cancelled,
    Expired,
    Crashed,
    Disconnected,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthDiagnosticDto {
    None,
    AwaitingInput,
    CandidateAccepted,
    CandidateRejected,
    Cancelled,
    Expired,
    Crashed,
    Disconnected,
    TimedOut,
    CleanupComplete,
}

impl AuthObservationDto {
    fn into_observation(self) -> Result<BrowserAuthObservation, ApiError> {
        if self.schema_version != BROWSER_AUTH_OBSERVATION_VERSION || self.sequence == 0 {
            return Err(ApiError::bad_request("AUTH_OBSERVATION_INVALID"));
        }
        Ok(BrowserAuthObservation {
            schema_version: self.schema_version,
            sequence: self.sequence,
            state: self.state.into(),
            diagnostic: self.diagnostic.into(),
        })
    }
}

impl From<AuthStateDto> for BrowserAuthState {
    fn from(value: AuthStateDto) -> Self {
        match value {
            AuthStateDto::Required => Self::Required,
            AuthStateDto::InputNeeded => Self::InputNeeded,
            AuthStateDto::CandidateReady => Self::CandidateReady,
            AuthStateDto::Cancelled => Self::Cancelled,
            AuthStateDto::Expired => Self::Expired,
            AuthStateDto::Crashed => Self::Crashed,
            AuthStateDto::Disconnected => Self::Disconnected,
            AuthStateDto::TimedOut => Self::TimedOut,
        }
    }
}

impl From<AuthDiagnosticDto> for BrowserAuthDiagnostic {
    fn from(value: AuthDiagnosticDto) -> Self {
        match value {
            AuthDiagnosticDto::None => Self::None,
            AuthDiagnosticDto::AwaitingInput => Self::AwaitingInput,
            AuthDiagnosticDto::CandidateAccepted => Self::CandidateAccepted,
            AuthDiagnosticDto::CandidateRejected => Self::CandidateRejected,
            AuthDiagnosticDto::Cancelled => Self::Cancelled,
            AuthDiagnosticDto::Expired => Self::Expired,
            AuthDiagnosticDto::Crashed => Self::Crashed,
            AuthDiagnosticDto::Disconnected => Self::Disconnected,
            AuthDiagnosticDto::TimedOut => Self::TimedOut,
            AuthDiagnosticDto::CleanupComplete => Self::CleanupComplete,
        }
    }
}

impl InputDto {
    fn into_input(self) -> Result<BrowserInput, ApiError> {
        match self {
            Self::Key { key } if bounded_text(&key, MAX_INPUT) => Ok(BrowserInput::Key { key }),
            Self::Pointer { x, y, button }
                if x.unsigned_abs() <= 10_000 && y.unsigned_abs() <= 10_000 =>
            {
                let button = match button.as_str() {
                    "primary" => PointerButton::Primary,
                    "secondary" => PointerButton::Secondary,
                    "auxiliary" => PointerButton::Auxiliary,
                    _ => return Err(ApiError::bad_request("AUTH_INPUT_INVALID")),
                };
                Ok(BrowserInput::Pointer { x, y, button })
            }
            Self::Text { value } if bounded_text(&value, MAX_INPUT) => {
                Ok(BrowserInput::Text { value })
            }
            Self::Submit => Ok(BrowserInput::Submit),
            _ => Err(ApiError::bad_request("AUTH_INPUT_INVALID")),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct EventsQuery {
    pub after: Option<u64>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthRouteStatus {
    Required,
    InputNeeded,
    CandidateReady,
    Cancelled,
    Expired,
    Crashed,
    Disconnected,
    TimedOut,
}

impl From<BrowserAuthState> for AuthRouteStatus {
    fn from(value: BrowserAuthState) -> Self {
        match value {
            BrowserAuthState::Required => Self::Required,
            BrowserAuthState::InputNeeded => Self::InputNeeded,
            BrowserAuthState::CandidateReady => Self::CandidateReady,
            BrowserAuthState::Cancelled => Self::Cancelled,
            BrowserAuthState::Expired => Self::Expired,
            BrowserAuthState::Crashed => Self::Crashed,
            BrowserAuthState::Disconnected => Self::Disconnected,
            BrowserAuthState::TimedOut => Self::TimedOut,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct StartResponse {
    pub request_id: String,
    pub attempt_id: String,
    pub state: AuthRouteStatus,
    pub expires_in_seconds: u64,
    pub duplicate: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct EventResponse {
    pub attempt_id: String,
    pub state: AuthRouteStatus,
    pub auth_events: Vec<AuthEventView>,
    pub browser_events: Vec<BrowserEventView>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AuthEventView {
    pub version: u16,
    pub sequence: u64,
    pub state: AuthRouteStatus,
    pub diagnostic: &'static str,
}

impl From<crate::browser_auth::BrowserAuthEvent> for AuthEventView {
    fn from(value: crate::browser_auth::BrowserAuthEvent) -> Self {
        Self {
            version: value.version,
            sequence: value.observation.sequence,
            state: value.observation.state.into(),
            diagnostic: diagnostic_name(value.observation.diagnostic),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct BrowserEventView {
    pub version: u16,
    pub sequence: u64,
    pub kind: &'static str,
}

impl From<BrowserEvent> for BrowserEventView {
    fn from(value: BrowserEvent) -> Self {
        Self {
            version: value.version,
            sequence: value.sequence,
            kind: event_name(&value.kind),
        }
    }
}

fn diagnostic_name(value: BrowserAuthDiagnostic) -> &'static str {
    match value {
        BrowserAuthDiagnostic::None => "none",
        BrowserAuthDiagnostic::AwaitingInput => "awaiting_input",
        BrowserAuthDiagnostic::CandidateAccepted => "candidate_accepted",
        BrowserAuthDiagnostic::CandidateRejected => "candidate_rejected",
        BrowserAuthDiagnostic::Cancelled => "cancelled",
        BrowserAuthDiagnostic::Expired => "expired",
        BrowserAuthDiagnostic::Crashed => "crashed",
        BrowserAuthDiagnostic::Disconnected => "disconnected",
        BrowserAuthDiagnostic::TimedOut => "timed_out",
        BrowserAuthDiagnostic::CleanupComplete => "cleanup_complete",
    }
}

fn event_name(value: &BrowserEventKind) -> &'static str {
    match value {
        BrowserEventKind::WorkerOpened { .. } => "worker_opened",
        BrowserEventKind::ProfileAttached => "profile_attached",
        BrowserEventKind::ProfileDetached => "profile_detached",
        BrowserEventKind::NavigationStarted { .. } => "navigation_started",
        BrowserEventKind::NavigationChanged { .. } => "navigation_changed",
        BrowserEventKind::ResourceObserved { .. } => "resource_observed",
        BrowserEventKind::Loading => "loading",
        BrowserEventKind::Ready => "ready",
        BrowserEventKind::InputAccepted { .. } => "input_accepted",
        BrowserEventKind::InputResult { .. } => "input_result",
        BrowserEventKind::NetworkDenied => "network_denied",
        BrowserEventKind::Error { .. } => "error",
        BrowserEventKind::OperationCancelled { .. } => "operation_cancelled",
        BrowserEventKind::WorkerClosed => "worker_closed",
        BrowserEventKind::WorkerCrashed => "worker_crashed",
        BrowserEventKind::WorkerTimedOut => "worker_timed_out",
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AcceptedResponse {
    pub attempt_id: String,
    pub accepted: bool,
    pub duplicate: bool,
    pub state: AuthRouteStatus,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct OperationResponse {
    pub attempt_id: String,
    pub operation_id: Option<u64>,
    pub duplicate: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ApiErrorBody {
    code: &'static str,
    message: &'static str,
}

#[derive(Clone, Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
}

impl ApiError {
    fn bad_request(code: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
        }
    }
    fn conflict(code: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
        }
    }
    fn not_found(code: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
        }
    }
    fn from_runtime(error: BrowserAuthRuntimeError) -> Self {
        let code = match error {
            BrowserAuthRuntimeError::Browser(error) => error.code(),
            BrowserAuthRuntimeError::Vault(VaultError::AccountNotFound) => "AUTH_ACCOUNT_NOT_FOUND",
            BrowserAuthRuntimeError::Vault(VaultError::CandidateNotFound) => {
                "AUTH_CANDIDATE_NOT_FOUND"
            }
            BrowserAuthRuntimeError::Vault(_) => "AUTH_VAULT_REJECTED",
            BrowserAuthRuntimeError::CandidateRejected => "AUTH_CANDIDATE_REJECTED",
            BrowserAuthRuntimeError::InvalidCandidate => "AUTH_CANDIDATE_INVALID",
            BrowserAuthRuntimeError::InvalidObservation => "AUTH_OBSERVATION_INVALID",
        };
        Self {
            status: StatusCode::CONFLICT,
            code,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                code: self.code,
                message: self.code,
            }),
        )
            .into_response()
    }
}

fn bounded_text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && value.chars().all(|c| !c.is_control())
}

fn bounded_ref(value: &str, max: usize) -> bool {
    bounded_text(value, max)
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".:_-".contains(c))
        && !value.to_ascii_lowercase().contains("token")
        && !value.to_ascii_lowercase().contains("cookie")
        && !value.to_ascii_lowercase().contains("authorization")
}

fn fingerprint<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn parse_navigation_url(value: &str) -> Result<Url, ApiError> {
    if !bounded_text(value, MAX_SOURCE) {
        return Err(ApiError::bad_request("AUTH_NAVIGATION_INVALID"));
    }
    let url = Url::parse(value).map_err(|_| ApiError::bad_request("AUTH_NAVIGATION_INVALID"))?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(ApiError::bad_request("AUTH_NAVIGATION_INVALID"));
    }
    Ok(url)
}

pub(crate) async fn start_handler(
    State(state): State<Arc<GatewayState>>,
    request: Result<Json<StartRequest>, JsonRejection>,
) -> Response {
    let Json(request) = match request {
        Ok(request) => request,
        Err(rejection) => return auth_json_rejection(rejection),
    };
    if !bounded_ref(&request.request_id, MAX_REQUEST_ID)
        || !bounded_ref(&request.site_id, MAX_SITE_ID)
        || !bounded_ref(&request.account_ref, MAX_ACCOUNT_REF)
    {
        return ApiError::bad_request("AUTH_REQUEST_INVALID").into_response();
    }
    match state
        .auth_routes
        .start(&request.request_id, &request.site_id, &request.account_ref)
        .await
    {
        Ok((attempt_id, state_value, expires, duplicate)) => Json(StartResponse {
            request_id: request.request_id,
            attempt_id,
            state: state_value.into(),
            expires_in_seconds: expires,
            duplicate,
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

pub(crate) async fn events_handler(
    State(state): State<Arc<GatewayState>>,
    Path(attempt_id): Path<String>,
    Query(query): Query<EventsQuery>,
) -> Response {
    if !bounded_ref(&attempt_id, 128) {
        return ApiError::not_found("AUTH_ATTEMPT_NOT_FOUND").into_response();
    }
    match state
        .auth_routes
        .events(&attempt_id, query.after.unwrap_or(0))
    {
        Ok((auth_events, browser_events, status)) => Json(EventResponse {
            attempt_id,
            state: status,
            auth_events,
            browser_events,
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

pub(crate) async fn navigation_handler(
    State(state): State<Arc<GatewayState>>,
    Path(attempt_id): Path<String>,
    request: Result<Json<NavigationRequest>, JsonRejection>,
) -> Response {
    let Json(request) = match request {
        Ok(request) => request,
        Err(rejection) => return auth_json_rejection(rejection),
    };
    if !bounded_ref(&attempt_id, 128) || !bounded_ref(&request.request_id, MAX_REQUEST_ID) {
        return ApiError::bad_request("AUTH_REQUEST_INVALID").into_response();
    }
    let url = match parse_navigation_url(&request.url) {
        Ok(url) => url,
        Err(error) => return error.into_response(),
    };
    let egress = state
        .egress_policy
        .read()
        .expect("egress policy poisoned")
        .clone();
    match state
        .auth_routes
        .navigate(
            &attempt_id,
            &request.request_id,
            request.operation_id,
            url,
            egress,
        )
        .await
    {
        Ok((operation_id, duplicate)) => Json(OperationResponse {
            attempt_id,
            operation_id: Some(operation_id),
            duplicate,
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

pub(crate) async fn input_handler(
    State(state): State<Arc<GatewayState>>,
    Path(attempt_id): Path<String>,
    request: Result<Json<InputRequest>, JsonRejection>,
) -> Response {
    let Json(request) = match request {
        Ok(request) => request,
        Err(rejection) => return auth_json_rejection(rejection),
    };
    if !bounded_ref(&attempt_id, 128) || !bounded_ref(&request.request_id, MAX_REQUEST_ID) {
        return ApiError::bad_request("AUTH_REQUEST_INVALID").into_response();
    }
    let input = match request.input.into_input() {
        Ok(input) => input,
        Err(error) => return error.into_response(),
    };
    match state
        .auth_routes
        .input(&attempt_id, &request.request_id, input)
        .await
    {
        Ok(duplicate) => Json(OperationResponse {
            attempt_id,
            operation_id: None,
            duplicate,
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

pub(crate) async fn cancel_handler(
    State(state): State<Arc<GatewayState>>,
    Path(attempt_id): Path<String>,
    request: Result<Json<CancelRequest>, JsonRejection>,
) -> Response {
    let Json(request) = match request {
        Ok(request) => request,
        Err(rejection) => return auth_json_rejection(rejection),
    };
    if !bounded_ref(&attempt_id, 128) || !bounded_ref(&request.request_id, MAX_REQUEST_ID) {
        return ApiError::bad_request("AUTH_REQUEST_INVALID").into_response();
    }
    match state.auth_routes.cancel(&attempt_id, &request.request_id) {
        Ok((_status, duplicate)) => Json(OperationResponse {
            attempt_id,
            operation_id: None,
            duplicate,
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

pub(crate) async fn candidate_handler(
    State(state): State<Arc<GatewayState>>,
    Path(attempt_id): Path<String>,
    request: Result<Json<CandidateRequest>, JsonRejection>,
) -> Response {
    let Json(request) = match request {
        Ok(request) => request,
        Err(rejection) => return auth_json_rejection(rejection),
    };
    if !bounded_ref(&attempt_id, 128)
        || !bounded_ref(&request.request_id, MAX_REQUEST_ID)
        || !bounded_ref(&request.candidate_session_id, MAX_CANDIDATE_REF)
        || !bounded_text(&request.source, MAX_SOURCE)
    {
        return ApiError::bad_request("AUTH_REQUEST_INVALID").into_response();
    }
    let locator = match state.source_sessions.recognize(&request.source) {
        Ok(locator) => locator,
        Err(_) => return ApiError::bad_request("AUTH_SOURCE_UNSUPPORTED").into_response(),
    };
    if request.operation_id == 0 {
        return ApiError::bad_request("AUTH_OPERATION_INVALID").into_response();
    }
    match state
        .auth_routes
        .candidate(&attempt_id, &request.request_id, &request, locator)
    {
        Ok((duplicate, status)) => Json(AcceptedResponse {
            attempt_id,
            accepted: true,
            duplicate,
            state: status,
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

pub(crate) async fn playback_handler(
    State(state): State<Arc<GatewayState>>,
    Path(attempt_id): Path<String>,
    request: Result<Json<CreateSessionRequest>, JsonRejection>,
) -> Response {
    let Json(request) = match request {
        Ok(request) => request,
        Err(rejection) => return auth_json_rejection(rejection),
    };
    if !bounded_ref(&attempt_id, 128)
        || !bounded_ref(&request.request_id, MAX_REQUEST_ID)
        || !bounded_text(&request.source, MAX_SOURCE)
        || !bounded_ref(&request.display_id, 128)
    {
        return ApiError::bad_request("AUTH_REQUEST_INVALID").into_response();
    }
    match state.auth_routes.playback(
        &GatewayService {
            state: Arc::clone(&state),
        },
        &attempt_id,
        request,
    ) {
        Ok(response) => response,
        Err(error) => error.into_response(),
    }
}

fn auth_json_rejection(rejection: JsonRejection) -> Response {
    let code = if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
        "AUTH_BODY_TOO_LARGE"
    } else {
        "AUTH_INVALID_JSON"
    };
    let status = if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
        StatusCode::PAYLOAD_TOO_LARGE
    } else {
        StatusCode::BAD_REQUEST
    };
    (
        status,
        Json(ApiErrorBody {
            code,
            message: code,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::SessionVault;

    #[test]
    fn dto_conversion_never_accepts_secret_like_reference() {
        assert!(!bounded_ref("cookie-session", MAX_CANDIDATE_REF));
        assert!(!bounded_ref("Bearer-token", MAX_CANDIDATE_REF));
        assert!(bounded_ref("candidate-1", MAX_CANDIDATE_REF));
    }

    #[test]
    fn event_projection_contains_only_generic_names() {
        let event = BrowserEvent {
            version: 1,
            sequence: 1,
            kind: BrowserEventKind::Ready,
        };
        let view = BrowserEventView::from(event);
        assert_eq!(view.kind, "ready");
    }

    #[tokio::test]
    async fn fake_coordinator_start_is_idempotent_and_cancel_cleans_attempt() {
        let vault = SessionVault::isolated_test();
        vault
            .register_account("fixture", "account", "fixture")
            .unwrap();
        let coordinator = AuthRouteCoordinator::fake_for_tests(vault);
        let first = coordinator
            .start("start-1", "fixture", "account")
            .await
            .unwrap();
        let replay = coordinator
            .start("start-1", "fixture", "account")
            .await
            .unwrap();
        assert_eq!(first.0, replay.0);
        assert!(!first.3);
        assert!(replay.3);
        let cancelled = coordinator.cancel(&first.0, "cancel-1").unwrap();
        assert!(!cancelled.1);
        assert!(coordinator.cancel(&first.0, "cancel-1").unwrap().1);
        assert!(matches!(
            coordinator.events(&first.0, 0),
            Err(ApiError {
                code: "AUTH_ATTEMPT_NOT_FOUND",
                ..
            })
        ));
    }
}
