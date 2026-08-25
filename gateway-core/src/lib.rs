use axum::body::Body;
use axum::extract::rejection::JsonRejection;
use axum::extract::{DefaultBodyLimit, Path, Query, Request, State};
use axum::http::header::{
    ACCEPT_RANGES, AUTHORIZATION, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG,
    HOST, IF_RANGE, LAST_MODIFIED, ORIGIN, RANGE,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use site_adapter_api::{ResolvedStream, ResolvedSubtitle, SiteAdapterRegistry, StreamProtocol};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use url::Url;
use uuid::Uuid;

pub mod auth;
pub mod browser;
pub mod control;
#[cfg(test)]
mod control_contract_tests;
pub mod control_view;
pub mod display_session;
pub mod security;
mod source_session;
pub use auth::{
    AccountState, AuthBoundaryError, CandidateValidation, CleanupResult, PendingIntent,
    PendingPlaybackAction, PendingSourceLocator, ScopedHttpResponse, ScopedSiteHttpClient,
    SessionSwapResult, SessionVault, SiteAccessContext, SiteAccount, SiteSessionRef, VaultError,
};
pub use control::{
    ControlCommand, ControlCommandError, ControlCommandRequest, ControlCommandResponse,
    ControlDisplaySnapshot, ControlErrorResponse, ControlEvent, ControlEventKind,
    ControlEventsResponse, ControlHandoffSnapshot, ControlItemSnapshot, ControlLookupError,
    ControlService, ControlSnapshot, ControlValidationError, DisplayPositionTelemetry,
};
pub use control_view::{
    ActionRequiredKind, ActionRequiredView, ActiveDisplayView, BrowserViewInput,
    ControlFreshnessView, ControlView, ControlViewInput, DisplayErrorInput, DisplayViewInput,
    NativePanelInput, NativePanelStatus, NativeSitePanelView, NowPlayingView, PendingActionView,
    PendingIntentInput, PlaybackContextView, PlaybackControlsView, PlaybackFreshnessView,
    PlaybackObservationView, SiteAccountStateView, SiteView, SiteViewInput,
};
pub use display_session::{
    DisplayCallback, DisplayCallbackResponse, DisplayContextResponse, DisplayHeartbeatResponse,
    DisplayRegistration, DisplayRegistrationResponse, DisplaySessionError,
    DisplaySessionErrorResponse, DisplaySessionService, LiveDisplayView, WebDisplayErrorCode,
    WebDisplayObservation,
};
pub use security::{
    EgressPolicy, EgressPolicyError, EgressScope, HttpAuthorityError, HttpAuthorityPolicy,
    SiteAccessCapability, SiteAccessError, ValidatedTarget,
};
pub use source_session::{
    CreateSessionErrorResponse, CreateSessionRequest, CreateSessionResponse, SessionMediaStream,
    SessionMediaView,
};

const MAX_MANIFEST_BYTES: usize = 512 * 1024;
const MAX_REDIRECTS: usize = 5;
const MAX_HTTP_BODY_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    pub session_id: String,
    pub item_id: String,
    pub item_revision: u64,
    pub resource_id: String,
}

impl Binding {
    pub fn new(session: &str, item: &str, item_revision: u64, resource: &str) -> Self {
        Self {
            session_id: session.into(),
            item_id: item.into(),
            item_revision,
            resource_id: resource.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpstreamResource {
    pub url: Url,
    pub protocol: StreamProtocol,
    pub public_headers: HeaderMap,
    pub secret_headers: HeaderMap,
    pub egress_scope: EgressScope,
}

#[derive(Clone, Debug)]
struct CapabilityRecord {
    binding: Binding,
    resource: UpstreamResource,
    expires_at: Instant,
    created_seq: u64,
}

#[derive(Debug, Eq, PartialEq)]
enum CapabilityError {
    NotFound,
    Expired,
    BindingMismatch,
}

#[derive(Debug)]
struct StoreInner {
    records: HashMap<String, CapabilityRecord>,
    order: VecDeque<(u64, String)>,
}

#[derive(Debug)]
struct CapabilityStore {
    inner: Mutex<StoreInner>,
    max_entries: usize,
    sequence: AtomicU64,
}

impl CapabilityStore {
    fn new(max_entries: usize) -> Self {
        assert!(max_entries > 0);
        Self {
            inner: Mutex::new(StoreInner {
                records: HashMap::new(),
                order: VecDeque::new(),
            }),
            max_entries,
            sequence: AtomicU64::new(1),
        }
    }

    fn issue(&self, binding: Binding, resource: UpstreamResource, ttl: Duration) -> String {
        let now = Instant::now();
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        let token = Uuid::new_v4().simple().to_string();
        let mut inner = self.inner.lock().expect("capability store poisoned");
        inner.records.retain(|_, record| record.expires_at > now);
        while inner.records.len() >= self.max_entries {
            if let Some((old_seq, old_token)) = inner.order.pop_front() {
                let should_remove = inner
                    .records
                    .get(&old_token)
                    .is_some_and(|record| record.created_seq == old_seq);
                if should_remove {
                    inner.records.remove(&old_token);
                }
            } else {
                break;
            }
        }
        inner.order.push_back((seq, token.clone()));
        inner.records.insert(
            token.clone(),
            CapabilityRecord {
                binding,
                resource,
                expires_at: now + ttl,
                created_seq: seq,
            },
        );
        token
    }

    fn get(&self, token: &str, binding: &Binding) -> Result<CapabilityRecord, CapabilityError> {
        let now = Instant::now();
        let mut inner = self.inner.lock().expect("capability store poisoned");
        let Some(record) = inner.records.get(token).cloned() else {
            return Err(CapabilityError::NotFound);
        };
        if record.expires_at <= now {
            inner.records.remove(token);
            return Err(CapabilityError::Expired);
        }
        if &record.binding != binding {
            return Err(CapabilityError::BindingMismatch);
        }
        Ok(record)
    }

    fn len(&self) -> usize {
        self.inner
            .lock()
            .expect("capability store poisoned")
            .records
            .len()
    }

    fn revoke(&self, token: &str) {
        self.inner
            .lock()
            .expect("capability store poisoned")
            .records
            .remove(token);
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ProofPaths {
    pub mp4_path: Option<String>,
    pub display_path: Option<String>,
    pub subtitle_path: Option<String>,
    pub hls_path: Option<String>,
    pub secret_path: Option<String>,
    pub chain: String,
}

#[derive(Clone)]
struct GatewayState {
    store: Arc<CapabilityStore>,
    egress_policy: Arc<RwLock<EgressPolicy>>,
    http_authorities: Arc<RwLock<HttpAuthorityPolicy>>,
    active_streams: Arc<AtomicUsize>,
    proof_paths: Arc<RwLock<ProofPaths>>,
    fixture_mp4: Arc<RwLock<Option<PathBuf>>>,
    fixture_vtt: Arc<RwLock<Option<PathBuf>>>,
    probe: Arc<ProbeStore>,
    control: ControlService,
    display_sessions: DisplaySessionService,
    source_sessions: source_session::SourceSessionService,
}

#[derive(Clone)]
pub struct GatewayService {
    state: Arc<GatewayState>,
}

#[derive(Debug)]
pub enum GatewayError {
    InvalidHeader,
    SecretHeader,
    InvalidSubtitle,
    UnsupportedSubtitleContentType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SubtitleTrackView {
    pub id: String,
    pub language: Option<String>,
    pub label: Option<String>,
    pub format: &'static str,
    pub gateway_path: String,
}

#[derive(Clone, Debug, Serialize)]
struct DisplayRenderingResponse {
    context: DisplayContextResponse,
    media: SessionMediaView,
}

const MAX_PROBE_RECORDS: usize = 256;

#[derive(Clone, Debug, Serialize)]
struct ProbeCommand {
    sequence: u64,
    request_id: String,
    kind: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct ProbeTelemetry {
    sequence: u64,
    kind: String,
    command_id: Option<String>,
    attempt_id: Option<String>,
    result: Option<String>,
    error_name: Option<String>,
    error_message: Option<String>,
    muted: Option<bool>,
    volume: Option<f64>,
    detail: Option<String>,
}

#[derive(Debug, Default)]
struct ProbeInner {
    next_command: u64,
    commands: VecDeque<ProbeCommand>,
    request_ids: HashMap<String, u64>,
    next_telemetry: u64,
    telemetry: VecDeque<ProbeTelemetry>,
}

#[derive(Debug, Default)]
struct ProbeStore {
    inner: Mutex<ProbeInner>,
}

#[derive(Clone, Debug, Deserialize)]
struct PlayCommandRequest {
    request_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ProbeEventsQuery {
    after: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
struct ProbeTelemetryRequest {
    kind: String,
    command_id: Option<String>,
    attempt_id: Option<String>,
    result: Option<String>,
    error_name: Option<String>,
    error_message: Option<String>,
    muted: Option<bool>,
    volume: Option<f64>,
    detail: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ProbeStateSnapshot {
    cursor: u64,
    commands: Vec<ProbeCommand>,
    telemetry: Vec<ProbeTelemetry>,
}

#[derive(Clone, Debug, Serialize)]
struct ProbeEventsResponse {
    cursor: u64,
    events: Vec<ProbeCommand>,
    truncated: bool,
}

#[derive(Clone, Debug, Serialize)]
struct PlayCommandResponse {
    accepted: bool,
    duplicate: bool,
    command: ProbeCommand,
}

impl ProbeStore {
    fn issue_play(&self, request_id: String) -> (ProbeCommand, bool) {
        let mut inner = self.inner.lock().expect("probe store poisoned");
        if let Some(sequence) = inner.request_ids.get(&request_id).copied()
            && let Some(command) = inner
                .commands
                .iter()
                .find(|command| command.sequence == sequence)
        {
            return (command.clone(), true);
        }

        inner.next_command += 1;
        let command = ProbeCommand {
            sequence: inner.next_command,
            request_id: request_id.clone(),
            kind: "play",
        };
        inner.request_ids.insert(request_id, command.sequence);
        inner.commands.push_back(command.clone());
        while inner.commands.len() > MAX_PROBE_RECORDS {
            if let Some(old) = inner.commands.pop_front() {
                inner.request_ids.remove(&old.request_id);
            }
        }
        (command, false)
    }

    fn state_snapshot(&self) -> ProbeStateSnapshot {
        let inner = self.inner.lock().expect("probe store poisoned");
        ProbeStateSnapshot {
            cursor: inner.next_command,
            commands: inner.commands.iter().cloned().collect(),
            telemetry: inner.telemetry.iter().cloned().collect(),
        }
    }

    fn events_after(&self, after: u64) -> ProbeEventsResponse {
        let inner = self.inner.lock().expect("probe store poisoned");
        let oldest = inner
            .commands
            .front()
            .map_or(inner.next_command + 1, |e| e.sequence);
        ProbeEventsResponse {
            cursor: inner.next_command,
            events: inner
                .commands
                .iter()
                .filter(|event| event.sequence > after)
                .cloned()
                .collect(),
            truncated: after + 1 < oldest,
        }
    }

    fn append_telemetry(&self, input: ProbeTelemetryRequest) -> u64 {
        let mut inner = self.inner.lock().expect("probe store poisoned");
        inner.next_telemetry += 1;
        let telemetry = ProbeTelemetry {
            sequence: inner.next_telemetry,
            kind: sanitize_text(&input.kind, 48),
            command_id: sanitize_optional(input.command_id, 128),
            attempt_id: sanitize_optional(input.attempt_id, 128),
            result: sanitize_optional(input.result, 48),
            error_name: sanitize_optional(input.error_name, 96),
            error_message: sanitize_optional(input.error_message, 256),
            muted: input.muted,
            volume: input.volume.filter(|value| value.is_finite()),
            detail: sanitize_optional(input.detail, 256),
        };
        inner.telemetry.push_back(telemetry);
        while inner.telemetry.len() > MAX_PROBE_RECORDS {
            inner.telemetry.pop_front();
        }
        inner.next_telemetry
    }

    fn reset(&self) {
        *self.inner.lock().expect("probe store poisoned") = ProbeInner::default();
    }
}

fn sanitize_optional(value: Option<String>, max: usize) -> Option<String> {
    value.map(|value| sanitize_text(&value, max))
}

fn sanitize_text(value: &str, max: usize) -> String {
    let mut output = String::new();
    let mut redact_next = false;
    for token in value.split_whitespace() {
        if redact_next {
            output.push_str("[redacted]");
            redact_next = false;
        } else if token.eq_ignore_ascii_case("bearer")
            || token.eq_ignore_ascii_case("basic")
            || token.eq_ignore_ascii_case("cookie:")
        {
            output.push_str(if token.eq_ignore_ascii_case("cookie:") {
                "[redacted-header]"
            } else {
                token
            });
            redact_next = true;
        } else if token.eq_ignore_ascii_case("cookie") {
            output.push_str("[redacted-header]");
            redact_next = true;
        } else {
            output.push_str(token);
        }
        output.push(' ');
    }
    output
        .trim_end()
        .chars()
        .filter(|character| !character.is_control())
        .take(max)
        .collect()
}

impl GatewayService {
    pub fn new(max_capabilities: usize) -> Self {
        Self::with_registry(max_capabilities, Arc::new(SiteAdapterRegistry::default()))
    }

    pub fn with_registry(max_capabilities: usize, registry: Arc<SiteAdapterRegistry>) -> Self {
        Self {
            state: Arc::new(GatewayState {
                store: Arc::new(CapabilityStore::new(max_capabilities)),
                egress_policy: Arc::new(RwLock::new(EgressPolicy::default())),
                http_authorities: Arc::new(RwLock::new(HttpAuthorityPolicy::default())),
                active_streams: Arc::new(AtomicUsize::new(0)),
                proof_paths: Arc::new(RwLock::new(ProofPaths {
                    chain: "SiteAdapterRegistry -> generic-direct -> ResolvedMedia -> MediaGateway -> WebDisplay".into(),
                    ..ProofPaths::default()
                })),
                fixture_mp4: Arc::new(RwLock::new(None)),
                fixture_vtt: Arc::new(RwLock::new(None)),
                probe: Arc::new(ProbeStore::default()),
                control: ControlService::default(),
                display_sessions: DisplaySessionService::default(),
                source_sessions: source_session::SourceSessionService::new(registry),
            }),
        }
    }

    /// Create a disposable session and current Web Display only for the
    /// gated deterministic browser harness. This is deliberately a Rust API,
    /// not a production route, and is unavailable in normal builds.
    #[cfg(feature = "control-ui-harness")]
    pub fn seed_control_ui_harness_session(&self) -> Result<String, DisplaySessionError> {
        let session_id = self.state.control.seed_harness_session(
            "control-ui-item",
            "control-ui-harness-media",
            "control-ui-display",
        );
        self.state.display_sessions.register(
            &self.state.control,
            DisplayRegistration {
                session_id: Some(session_id.clone()),
                display_id: "control-ui-display".into(),
                label: "Control UI harness display".into(),
                capabilities: vec!["video".into(), "audio".into(), "seek".into()],
                previous_registration_id: None,
                previous_lease_token: None,
            },
        )?;
        Ok(session_id)
    }

    /// Register a private integration from deployment/admin configuration.
    /// User/plugin URLs cannot create this exception because validation also
    /// requires the named entry and its configured origin.
    pub fn configure_local_service(
        &self,
        name: impl Into<String>,
        origin: Url,
    ) -> Result<(), EgressPolicyError> {
        self.state
            .egress_policy
            .write()
            .expect("egress policy poisoned")
            .configure_local_service(name, &origin)
    }

    /// Configure the exact deployment authority accepted by the HTTP/control
    /// surface. This is deployment-owned; request Host/Origin values cannot
    /// create or widen the authority set.
    pub fn configure_http_authority(&self, origin: Url) -> Result<(), HttpAuthorityError> {
        self.state
            .http_authorities
            .write()
            .expect("http authority policy poisoned")
            .configure(&origin)
    }

    pub fn resource_from_resolved(
        stream: &ResolvedStream,
        scope: EgressScope,
    ) -> Result<UpstreamResource, GatewayError> {
        let mut public_headers = HeaderMap::new();
        for (name, value) in &stream.public_headers {
            if security::is_secret_header(name, value) {
                return Err(GatewayError::SecretHeader);
            }
            let name =
                HeaderName::from_bytes(name.as_bytes()).map_err(|_| GatewayError::InvalidHeader)?;
            let value = HeaderValue::from_str(value).map_err(|_| GatewayError::InvalidHeader)?;
            public_headers.insert(name, value);
        }
        Ok(UpstreamResource {
            url: stream.url.clone(),
            protocol: stream.protocol,
            public_headers,
            secret_headers: HeaderMap::new(),
            egress_scope: scope,
        })
    }

    /// Bind subtitle metadata to the same opaque Gateway capability mechanism
    /// used by media. The browser receives only the resulting same-origin
    /// path; upstream URL, headers, and access references remain server-side.
    pub fn subtitle_track_view(
        &self,
        subtitle: &ResolvedSubtitle,
        binding: Binding,
        scope: EgressScope,
        ttl: Duration,
    ) -> Result<SubtitleTrackView, GatewayError> {
        if !matches!(subtitle.url.scheme(), "http" | "https") {
            return Err(GatewayError::InvalidSubtitle);
        }
        if !subtitle.content_type.eq_ignore_ascii_case("text/vtt") {
            return Err(GatewayError::UnsupportedSubtitleContentType);
        }
        let mut public_headers = HeaderMap::new();
        for (name, value) in &subtitle.public_headers {
            if security::is_secret_header(name, value) {
                return Err(GatewayError::SecretHeader);
            }
            let name =
                HeaderName::from_bytes(name.as_bytes()).map_err(|_| GatewayError::InvalidHeader)?;
            let value = HeaderValue::from_str(value).map_err(|_| GatewayError::InvalidHeader)?;
            public_headers.insert(name, value);
        }
        let gateway_path = self.issue_path(
            binding,
            UpstreamResource {
                url: subtitle.url.clone(),
                protocol: StreamProtocol::HttpFile,
                public_headers,
                secret_headers: HeaderMap::new(),
                egress_scope: scope,
            },
            ttl,
        );
        Ok(SubtitleTrackView {
            id: subtitle.id.clone(),
            language: subtitle.language.clone(),
            label: subtitle.label.clone(),
            format: "webvtt",
            gateway_path,
        })
    }

    pub fn issue_path(
        &self,
        binding: Binding,
        resource: UpstreamResource,
        ttl: Duration,
    ) -> String {
        self.issue_path_with_token(binding, resource, ttl).0
    }

    pub(crate) fn issue_path_with_token(
        &self,
        binding: Binding,
        resource: UpstreamResource,
        ttl: Duration,
    ) -> (String, String) {
        let token = self.state.store.issue(binding.clone(), resource, ttl);
        (stream_path(&token, &binding), token)
    }

    pub(crate) fn revoke_capability(&self, token: &str) {
        self.state.store.revoke(token);
    }

    pub fn configure_proof_paths(&self, paths: ProofPaths) {
        *self
            .state
            .proof_paths
            .write()
            .expect("proof paths poisoned") = paths;
    }

    pub fn configure_fixture_mp4(&self, path: Option<PathBuf>) {
        *self
            .state
            .fixture_mp4
            .write()
            .expect("fixture path poisoned") = path;
    }

    pub fn configure_fixture_vtt(&self, path: Option<PathBuf>) {
        *self
            .state
            .fixture_vtt
            .write()
            .expect("subtitle fixture path poisoned") = path;
    }

    pub fn active_streams(&self) -> usize {
        self.state.active_streams.load(Ordering::SeqCst)
    }

    pub fn capability_count(&self) -> usize {
        self.state.store.len()
    }

    pub fn max_capabilities(&self) -> usize {
        self.state.store.max_entries
    }

    #[cfg(test)]
    pub(crate) fn control(&self) -> ControlService {
        self.state.control.clone()
    }

    pub(crate) fn create_session(
        &self,
        request: CreateSessionRequest,
    ) -> source_session::CreationOutcome {
        self.state.source_sessions.create(
            self,
            &self.state.control,
            &self.state.display_sessions,
            request,
        )
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(entry_handler))
            .route(
                "/stream/{token}/{session}/{item}/{revision}/{resource}",
                get(stream_handler).head(stream_handler),
            )
            .route("/metrics", get(metrics_handler))
            .route("/proof/paths", get(proof_paths_handler))
            .route("/display", get(display_handler))
            .route("/control", get(control_handler))
            .route("/api/v1/control/{session_id}", get(control_view_handler))
            .route(
                "/api/v1/sessions/{session_id}",
                get(control_session_snapshot_handler),
            )
            .route("/api/v1/sessions", post(create_session_handler))
            .route(
                "/api/v1/sessions/{session_id}/commands",
                post(control_session_command_handler),
            )
            .route(
                "/api/v1/sessions/{session_id}/events",
                get(control_session_events_handler),
            )
            .route("/api/v1/displays/register", post(display_register_handler))
            .route("/api/v1/displays", get(live_displays_handler))
            .route(
                "/api/v1/displays/{display_id}/heartbeat",
                post(display_heartbeat_handler),
            )
            .route(
                "/api/v1/displays/{display_id}/context",
                get(display_context_handler),
            )
            .route(
                "/api/v1/displays/{display_id}/rendering",
                get(display_rendering_handler),
            )
            .route(
                "/api/v1/displays/{display_id}/callback",
                post(display_callback_handler),
            )
            .route("/api/v1/display-probe/state", get(probe_state_handler))
            .route("/api/v1/display-probe/events", get(probe_events_handler))
            .route(
                "/api/v1/display-probe/commands",
                post(probe_command_handler),
            )
            .route(
                "/api/v1/display-probe/telemetry",
                post(probe_telemetry_handler),
            )
            .route("/api/v1/display-probe/reset", post(probe_reset_handler))
            .route("/secret-display", get(secret_display_handler))
            .route(
                "/fixture/protected.mp4",
                get(fixture_handler).head(fixture_handler),
            )
            .route(
                "/fixture/subtitles.vtt",
                get(subtitle_fixture_handler).head(subtitle_fixture_handler),
            )
            .route("/healthz", get(|| async { "ok" }))
            .layer(DefaultBodyLimit::max(MAX_HTTP_BODY_BYTES))
            .layer(middleware::from_fn_with_state(
                self.state.clone(),
                http_surface_guard,
            ))
            .with_state(self.state.clone())
    }
}

async fn http_surface_guard(
    State(state): State<Arc<GatewayState>>,
    request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    let Some(host) = headers.get(HOST).and_then(|value| value.to_str().ok()) else {
        return (StatusCode::BAD_REQUEST, "invalid_host").into_response();
    };
    let http_authorities = state
        .http_authorities
        .read()
        .expect("http authority policy poisoned")
        .clone();
    if !security::is_valid_http_host(host) {
        return (StatusCode::BAD_REQUEST, "invalid_host").into_response();
    }
    if !http_authorities.allows_host(host) {
        return (StatusCode::MISDIRECTED_REQUEST, "host_not_allowed").into_response();
    }

    if let Some(origin) = headers.get(ORIGIN).and_then(|value| value.to_str().ok())
        && !http_authorities.allows_origin_for_host(origin, host)
    {
        return (StatusCode::FORBIDDEN, "origin_mismatch").into_response();
    }

    if matches!(
        request.method(),
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE
    ) {
        let is_json = headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value.split(';').next().is_some_and(|media_type| {
                    media_type.trim().eq_ignore_ascii_case("application/json")
                })
            });
        if !is_json {
            return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "content_type_required").into_response();
        }
        let Some(origin) = headers.get(ORIGIN).and_then(|value| value.to_str().ok()) else {
            return (StatusCode::FORBIDDEN, "origin_required").into_response();
        };
        if !http_authorities.allows_origin_for_host(origin, host) {
            return (StatusCode::FORBIDDEN, "origin_mismatch").into_response();
        }
    }

    next.run(request).await
}

fn stream_path(token: &str, binding: &Binding) -> String {
    format!(
        "/stream/{}/{}/{}/{}/{}",
        token, binding.session_id, binding.item_id, binding.item_revision, binding.resource_id
    )
}

#[derive(Serialize)]
struct Metrics {
    active_streams: usize,
    capability_count: usize,
    capability_limit: usize,
}

async fn metrics_handler(State(state): State<Arc<GatewayState>>) -> Json<Metrics> {
    Json(Metrics {
        active_streams: state.active_streams.load(Ordering::SeqCst),
        capability_count: state.store.len(),
        capability_limit: state.store.max_entries,
    })
}

async fn proof_paths_handler(State(state): State<Arc<GatewayState>>) -> Json<ProofPaths> {
    Json(
        state
            .proof_paths
            .read()
            .expect("proof paths poisoned")
            .clone(),
    )
}

async fn entry_handler() -> Response {
    Html(ENTRY_PAGE.to_string()).into_response()
}

#[derive(Clone, Debug, Deserialize)]
struct DisplayPageQuery {
    profile: Option<String>,
}

async fn display_handler(
    State(state): State<Arc<GatewayState>>,
    Query(query): Query<DisplayPageQuery>,
) -> Response {
    let paths = state
        .proof_paths
        .read()
        .expect("proof paths poisoned")
        .clone();
    if query.profile.as_deref() == Some("tv") {
        return tv_display_page(paths.display_path, paths.subtitle_path);
    }
    let path = paths.display_path.or(paths.mp4_path);
    probe_display_page(path)
}

async fn secret_display_handler(State(state): State<Arc<GatewayState>>) -> Response {
    let path = state
        .proof_paths
        .read()
        .expect("proof paths poisoned")
        .secret_path
        .clone();
    video_page(path, "R001 secret-boundary fixture")
}

fn video_page(path: Option<String>, title: &str) -> Response {
    let Some(path) = path else {
        return (StatusCode::NOT_FOUND, "proof media is not configured").into_response();
    };
    Html(format!(
        "<!doctype html><meta charset=\"utf-8\"><title>{title}</title><video id=\"player\" controls muted preload=\"auto\" src=\"{path}\"></video>"
    ))
    .into_response()
}

fn probe_display_page(path: Option<String>) -> Response {
    let Some(path) = path else {
        return (StatusCode::NOT_FOUND, "probe media is not configured").into_response();
    };
    Html(DISPLAY_PAGE.replace("__MEDIA_PATH__", &escape_html_attribute(&path))).into_response()
}

fn tv_display_page(media_path: Option<String>, subtitle_path: Option<String>) -> Response {
    let media_attribute = media_path
        .as_deref()
        .map(|path| format!("src=\"{}\"", escape_html_attribute(path)))
        .unwrap_or_default();
    let subtitle_attribute = subtitle_path
        .as_deref()
        .map(|path| format!("src=\"{}\"", escape_html_attribute(path)))
        .unwrap_or_default();
    Html(
        TV_DISPLAY_PAGE
            .replace("__MEDIA_ATTRIBUTE__", &media_attribute)
            .replace("__SUBTITLE_ATTRIBUTE__", &subtitle_attribute),
    )
    .into_response()
}

const ENTRY_PAGE: &str = r##"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Web Media Gateway</title>
<style>:root{font-family:system-ui,sans-serif;color:#172033;background:#f5f7fb}main{max-width:42rem;margin:12vh auto;padding:2rem;background:white;border:1px solid #d9e0ee;border-radius:1rem;box-shadow:0 1rem 3rem #10204012}h1{margin-top:0}nav{display:grid;grid-template-columns:1fr 1fr;gap:1rem}a{display:block;padding:1.2rem;border-radius:.6rem;background:#1e3a68;color:white;text-align:center;text-decoration:none;font-weight:700}a:focus-visible{outline:4px solid #f5b942;outline-offset:3px}#countdown{min-height:1.5rem;color:#52627b}</style></head>
<body><main><h1>Web Media Gateway</h1><p>Choose where to play. TV Display is the default after five seconds when there is no explicit choice.</p><nav><a id="tv" href="/display?profile=tv">TV Display</a><a id="control" href="/control">Control</a></nav><p id="countdown" role="status" aria-live="polite">TV Display in 5 seconds.</p></main>
<script>(()=>{let remaining=5;let cancelled=false;const status=document.querySelector('#countdown');const cancel=()=>{if(cancelled)return;cancelled=true;status.textContent='Choose a mode above.';};document.addEventListener('pointerdown',cancel,{once:true});document.addEventListener('keydown',cancel,{once:true});const timer=setInterval(()=>{if(cancelled){clearInterval(timer);return;}remaining-=1;if(remaining>0)status.textContent=`TV Display in ${remaining} second${remaining===1?'':'s'}.`;else{clearInterval(timer);window.location.assign('/display?profile=tv');}},1000);})();</script></body></html>"##;

const TV_DISPLAY_PAGE: &str = r##"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover"><title>TV Web Display</title>
<style>:root{color-scheme:dark;font-family:system-ui,sans-serif}*{box-sizing:border-box}html,body{width:100%;height:100%;margin:0;background:#000}body{overflow:hidden}#display-shell{position:relative;width:100vw;height:100vh;min-height:100dvh;background:#000}#player{width:100%;height:100%;object-fit:contain;background:#000}#overlay{position:absolute;inset:auto 3vw 3vh;max-width:48rem;padding:1rem 1.25rem;border:1px solid #65718a;border-radius:.7rem;background:#101827ee}#overlay.playing{opacity:0;pointer-events:none;transition:opacity .35s}#overlay:hover,#overlay:focus-within{opacity:1;pointer-events:auto}h1{font-size:clamp(1.1rem,2.4vw,2rem);margin:.1rem 0 .4rem}p{margin:.35rem 0}button{min-height:2.8rem;margin:.35rem .35rem .2rem 0;padding:.5rem .8rem;color:#fff;background:#263b62;border:1px solid #8aa9e8;border-radius:.35rem;font:inherit}button:focus-visible{outline:3px solid #ffca55;outline-offset:2px}#diagnostics{max-height:11rem;overflow:auto;white-space:pre-wrap;font-size:.75rem;color:#b8c5d8}.ok{color:#b8f0bd}.warn{color:#ffd58a}</style></head>
<body><main id="display-shell" class="viewport-immersive"><video id="player" playsinline preload="auto" __MEDIA_ATTRIBUTE__><track id="subtitle-track" kind="subtitles" default __SUBTITLE_ATTRIBUTE__></video><section id="overlay" aria-live="polite"><h1>TV Web Display</h1><p id="status">Waiting for a Gateway playback session.</p><p id="capabilities"></p><button id="activate" type="button">Press OK to play</button><button id="fullscreen" type="button">Try Fullscreen</button><button id="retry" type="button">Reconnect Display</button><details><summary>Display diagnostics</summary><pre id="diagnostics">starting…</pre></details></section></main>
<script>(()=>{
  const player=document.querySelector('#player'), shell=document.querySelector('#display-shell'), overlay=document.querySelector('#overlay'), status=document.querySelector('#status'), diagnostics=document.querySelector('#diagnostics'), capabilities=document.querySelector('#capabilities'), track=document.querySelector('#subtitle-track');
  const storageKey='gateway.tv.display.v1'; let registration=null, heartbeatTimer=null, renderingTimer=null, reconnecting=false, mediaError=null;
  const safeError=e=>({name:String(e?.name||'UnknownError').slice(0,96),message:String(e?.message||'').replace(/https?:\/\/[^\s]+/g,'[url-redacted]').replace(/(bearer|cookie|authorization)\s*[:=]?\s*[^\s]+/ig,'$1 [redacted]').slice(0,180)});
  const show=(message,kind='')=>{status.textContent=message;status.className=kind;diagnostics.textContent=`${new Date().toISOString()} ${message}\n`+diagnostics.textContent.slice(0,1800)};
  const readSaved=()=>{try{const value=JSON.parse(sessionStorage.getItem(storageKey)||'null');return value&&typeof value==='object'?value:null}catch(_){return null}};
  const save=()=>{if(!registration)return;try{sessionStorage.setItem(storageKey,JSON.stringify({display_id:registration.display_id,registration_id:registration.registration_id,lease_token:registration.lease_token}))}catch(_){}};
  const displayId=()=>{const old=readSaved();if(old?.display_id&&/^[A-Za-z0-9._:-]{1,128}$/.test(old.display_id))return old.display_id;return `tv-display-${crypto.randomUUID?crypto.randomUUID().replaceAll('-','').slice(0,20):Math.random().toString(36).slice(2,18)}`};
  const sendCallback=async(errorCode)=>{if(!registration?.context)return;const c=registration.context;await fetch(`/api/v1/displays/${encodeURIComponent(registration.display_id)}/callback`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({lease_token:registration.lease_token,session_id:c.session_id,item_id:c.item_id,item_revision:c.item_revision,session_revision:c.session_revision,display_id:registration.display_id,display_generation:c.display_generation,observation:null,error_code:errorCode})}).catch(()=>{});};
  const renderContext=()=>{if(registration?.context){show(`Session ready: ${registration.context.state}.`,'ok')}else show('Waiting for a Gateway playback session.','warn')};
  const fetchRendering=async()=>{if(!registration)return;try{const response=await fetch(`/api/v1/displays/${encodeURIComponent(registration.display_id)}/rendering`,{headers:{'x-display-lease':registration.lease_token},cache:'no-store'});if(response.status===404){registration.context=null;renderContext();return}if(!response.ok)throw new Error(`rendering ${response.status}`);const rendering=await response.json();registration.context=rendering.context;const stream=rendering.media?.streams?.find(item=>item.protocol==='http_file')||rendering.media?.streams?.[0];if(stream?.gateway_path&&player.src!==new URL(stream.gateway_path,location.href).href){player.src=stream.gateway_path;player.load()}capabilities.textContent=`Gateway media ready: ${rendering.media.title||'current item'}`;renderContext()}catch(error){show(`Display rendering reconnecting (${safeError(error).name}).`,'warn')}};
  const heartbeat=()=>{if(heartbeatTimer)clearInterval(heartbeatTimer);if(!registration)return;const delay=Math.max(1000,Math.floor(registration.lease_ttl_ms/3));heartbeatTimer=setInterval(async()=>{try{const response=await fetch(`/api/v1/displays/${encodeURIComponent(registration.display_id)}/heartbeat`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({lease_token:registration.lease_token})});if(!response.ok)throw new Error(`heartbeat ${response.status}`);await fetchRendering();show(registration.context?'Display connected; session context retained.':'Display connected; waiting for a session.','ok')}catch(error){show(`Display heartbeat retrying (${safeError(error).name}).`,'warn')}},delay)};
  const register=async()=>{const old=readSaved();const body={display_id:displayId(),label:'TV Web Display',capabilities:['video','audio','subtitles'],previous_registration_id:old?.registration_id||null,previous_lease_token:old?.lease_token||null};try{const response=await fetch('/api/v1/displays/register',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(body)});if(!response.ok){if(old){sessionStorage.removeItem(storageKey);return register()}throw new Error(`register ${response.status}`)}registration=await response.json();save();capabilities.textContent='Gateway registration active; subtitles: '+(registration.context?'ready':'advertised');renderContext();await fetchRendering();heartbeat();if(renderingTimer)clearInterval(renderingTimer);renderingTimer=setInterval(fetchRendering,1000);return true}catch(error){registration=null;show(`Display reconnect waiting (${safeError(error).name}).`,'warn');return false}};
  const play=async()=>{if(!player.getAttribute('src')){show('No Gateway media is attached yet; waiting for a session.','warn');return}overlay.classList.remove('playing');try{await player.play();show('Playback started.','ok');await sendCallback(null);overlay.classList.add('playing')}catch(error){mediaError=safeError(error);show(`Playback could not start (${mediaError.name}); use OK after interaction.`,'warn');await sendCallback('command_rejected')}};
  document.querySelector('#activate').addEventListener('click',play);
  document.querySelector('#retry').addEventListener('click',()=>{if(!reconnecting){reconnecting=true;register().finally(()=>{reconnecting=false})}});
  document.querySelector('#fullscreen').addEventListener('click',async()=>{if(!document.documentElement.requestFullscreen){show('Fullscreen unavailable; viewport immersive mode remains active.','warn');return}try{await document.documentElement.requestFullscreen();show('Fullscreen enabled.','ok')}catch(error){show(`Fullscreen rejected (${safeError(error).name}); viewport immersive mode remains active.`,'warn')}});
  document.addEventListener('keydown',event=>{if((event.key==='Enter'||event.key==='OK')&&document.activeElement instanceof HTMLButtonElement)document.activeElement.click()});
  player.addEventListener('error',()=>show('Gateway media reported an error; waiting for recovery.','warn'));
  const secure=window.isSecureContext?'secure context enhancements available':'HTTP baseline: secure-context enhancements optional';
  const wake='wakeLock' in navigator?'Wake Lock available':'Wake Lock unavailable';
  const sw='serviceWorker' in navigator?'Service Worker available':'Service Worker unavailable';
  show(`${secure}. ${wake}; ${sw}.`); register();
  if(track.track) track.track.mode='hidden';
  if(track.getAttribute('src')==='')track.remove();
  window.__displayPrep={getRegistration:()=>registration,getStatus:()=>status.textContent,play,reconnect:register,getRendering:()=>registration?.context||null};
  window.addEventListener('pagehide',()=>{if(heartbeatTimer)clearInterval(heartbeatTimer);if(renderingTimer)clearInterval(renderingTimer)});
})();</script></body></html>"##;

fn escape_html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const DISPLAY_PAGE: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">
  <title>Web Display Remote Playback Probe</title>
  <style>
    :root { color-scheme: dark; font-family: system-ui,sans-serif; }
    * { box-sizing: border-box; }
    html, body { width:100%; height:100%; margin:0; background:#000; }
    body { overflow:hidden; }
    #display-shell { position:relative; width:100vw; height:100vh; min-height:100dvh; background:#000; }
    #player { width:100%; height:100%; object-fit:contain; background:#000; }
    #overlay { position:absolute; inset:auto 3vw 3vh; max-width:42rem; padding:1rem 1.2rem; border:1px solid #555; border-radius:.7rem; background:#111d; }
    #overlay.playing { opacity:0; pointer-events:none; transition:opacity .4s; }
    #overlay:hover, #overlay:focus-within { opacity:1; pointer-events:auto; }
    h1 { font-size:clamp(1.1rem,2.4vw,2rem); margin:.1rem 0 .4rem; }
    p { margin:.35rem 0; }
    button { min-height:2.8rem; margin:.35rem .35rem .2rem 0; padding:.5rem .8rem; color:#fff; background:#263b62; border:1px solid #8aa9e8; border-radius:.35rem; font:inherit; }
    button:focus-visible { outline:3px solid #ffca55; outline-offset:2px; }
    #diagnostics { max-height:12rem; overflow:auto; white-space:pre-wrap; font-size:.75rem; color:#b8c5d8; }
  </style>
</head>
<body>
  <main id="display-shell" class="viewport-immersive">
    <video id="player" playsinline preload="auto" src="__MEDIA_PATH__"></video>
    <section id="overlay" aria-live="polite">
      <h1>Remote playback ready</h1>
      <p id="status">Waiting for a remote play command.</p>
      <button id="activate" type="button">Press OK to enable remote playback</button>
      <button id="fullscreen" type="button">Try Fullscreen</button>
      <button id="reset" type="button">Reset / reload probe</button>
      <details><summary>Probe diagnostics</summary><pre id="diagnostics">starting…</pre></details>
    </section>
  </main>
  <script>
    (() => {
      const player = document.querySelector('#player');
      const overlay = document.querySelector('#overlay');
      const status = document.querySelector('#status');
      const diagnostics = document.querySelector('#diagnostics');
      const seenCommands = new Set();
      let cursor = 0;
      let transport = 'disconnected';

      const safeError = error => ({
        name: String(error?.name || 'UnknownError').slice(0, 96),
        message: String(error?.message || 'No browser error message').replace(/https?:\/\/[^\s]+/g, '[url-redacted]').slice(0, 256)
      });
      const attemptId = prefix => `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
      const send = payload => fetch('/api/v1/display-probe/telemetry', {
        method: 'POST', headers: {'content-type': 'application/json'}, body: JSON.stringify(payload), keepalive: true
      }).catch(() => {});
      const show = text => {
        status.textContent = text;
        diagnostics.textContent = `${new Date().toISOString()} ${text}\n` + diagnostics.textContent.slice(0, 1800);
      };

      async function playOnce(source, commandId = null) {
        const id = attemptId(source);
        if (player.ended) player.currentTime = 0;
        player.muted = false;
        player.volume = 1;
        show(`${source} audible play attempt ${id}…`);
        let result = 'reject';
        let error = {};
        try {
          const playPromise = player.play();
          await playPromise;
          result = 'resolve';
          show(`${source} audible play resolved; muted=${player.muted} volume=${player.volume}`);
        } catch (caught) {
          error = safeError(caught);
          overlay.classList.remove('playing');
          show(`${source} audible play rejected: ${error.name}`);
        }
        await send({
          kind: 'play_attempt', attempt_id: id, command_id: commandId, result,
          error_name: error.name || null, error_message: error.message || null,
          muted: player.muted, volume: player.volume, detail: `source=${source}`
        });
        return result;
      }

      async function poll() {
        try {
          const response = await fetch(`/api/v1/display-probe/events?after=${cursor}`, {cache:'no-store'});
          if (!response.ok) throw new Error(`probe events HTTP ${response.status}`);
          const payload = await response.json();
          if (payload.truncated) {
            const snapshot = await (await fetch('/api/v1/display-probe/state', {cache:'no-store'})).json();
            cursor = snapshot.cursor;
          } else {
            cursor = payload.cursor;
            for (const event of payload.events) {
              if (event.kind === 'play' && !seenCommands.has(event.sequence)) {
                seenCommands.add(event.sequence);
                await playOnce('remote', event.request_id);
              }
            }
          }
          if (transport !== 'connected') {
            transport = 'connected';
            send({kind:'transport', result:'connected', detail:'remote event polling connected'});
            show('Remote event connection ready.');
          }
        } catch (error) {
          if (transport !== 'reconnecting') {
            transport = 'reconnecting';
            send({kind:'transport', result:'reconnecting', error_name:safeError(error).name, error_message:safeError(error).message});
          }
          show('Remote event connection retrying…');
        }
      }

      document.querySelector('#activate').addEventListener('click', async () => {
        const result = await playOnce('activation');
        if (result === 'resolve') {
          show('Activation completed. Remote playback may now be retried.');
          player.pause();
        }
      });
      document.querySelector('#fullscreen').addEventListener('click', async () => {
        if (!document.documentElement.requestFullscreen) {
          send({kind:'fullscreen', result:'unavailable', detail:'requestFullscreen is not exposed'});
          show('Fullscreen unavailable; viewport immersive mode remains active.');
          return;
        }
        try {
          await document.documentElement.requestFullscreen();
          send({kind:'fullscreen', result:'resolve'});
          show('Fullscreen enabled.');
        } catch (error) {
          const safe = safeError(error);
          send({kind:'fullscreen', result:'reject', error_name:safe.name, error_message:safe.message});
          show(`Fullscreen rejected (${safe.name}); viewport immersive mode remains active.`);
        }
      });
      document.querySelector('#reset').addEventListener('click', async () => {
        await fetch('/api/v1/display-probe/reset', {method:'POST'}).catch(() => {});
        window.location.reload();
      });
      player.addEventListener('ended', () => { overlay.classList.remove('playing'); send({kind:'media', result:'ended'}); show('Playback ended; waiting for another remote command.'); });
      player.addEventListener('error', () => { overlay.classList.remove('playing'); send({kind:'media', result:'error', detail:'HTMLMediaElement error'}); show('Media element reported an error.'); });
      document.addEventListener('visibilitychange', () => send({kind:'visibility', result:document.visibilityState}));
      window.addEventListener('pageshow', () => send({kind:'lifecycle', result:'pageshow'}));
      window.addEventListener('pagehide', () => send({kind:'lifecycle', result:'pagehide'}));
      window.addEventListener('online', () => { transport = 'reconnecting'; send({kind:'network', result:'online'}); });
      window.addEventListener('offline', () => { transport = 'reconnecting'; send({kind:'network', result:'offline'}); });
      document.addEventListener('fullscreenchange', () => send({kind:'fullscreen', result:document.fullscreenElement ? 'active' : 'inactive'}));
      document.addEventListener('keydown', event => {
        if (event.key === 'Enter' || event.key === 'OK') document.querySelector('#activate').click();
      });
      window.__r002Probe = { poll, playOnce, getCursor: () => cursor };
      (async () => {
        const snapshot = await (await fetch('/api/v1/display-probe/state', {cache:'no-store'})).json();
        cursor = snapshot.cursor;
        send({kind:'lifecycle', result:'ready', detail:`cursor=${cursor}`});
        await poll();
        window.setInterval(poll, 1000);
      })().catch(error => show(`Probe initialization failed: ${safeError(error).name}`));
    })();
  </script>
</body>
</html>"##;

const CONTROL_PAGE: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Control</title>
<style>
:root{color-scheme:light dark;font:16px system-ui,sans-serif}body{margin:0;background:#10131a;color:#f4f6fb}
main{max-width:52rem;margin:0 auto;padding:1.25rem}section{margin:1rem 0;padding:1rem;border:1px solid #3c4658;border-radius:.75rem;background:#171c26}
h1,h2{margin:.1rem 0 .65rem}p{margin:.45rem 0;color:#c8d1df}dl{display:grid;grid-template-columns:minmax(8rem,auto) 1fr;gap:.4rem .8rem;margin:.5rem 0}dt{color:#9eabc0}dd{margin:0}
.controls{display:flex;flex-wrap:wrap;gap:.5rem}button{min-height:2.8rem;padding:.6rem .9rem;border:1px solid #8da8d8;border-radius:.4rem;background:#2b4778;color:#fff;font:inherit}button:disabled{opacity:.45;cursor:not-allowed}
input{max-width:9rem;padding:.65rem;border:1px solid #66748b;border-radius:.4rem;background:#0e1219;color:inherit;font:inherit}.status{min-height:1.5rem}.error{color:#ffb6b6}.sr-only{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
</style></head>
<body><main>
<h1>Control</h1><p id="connection" class="status" role="status">Starting…</p>
<section id="source-entry" aria-labelledby="source-heading"><h2 id="source-heading">Start playback</h2><p>Choose a live Web Display and submit a bounded direct media URL.</p><form id="source-form"><label for="display-selector">Display</label><select id="display-selector" required><option value="">Loading live displays…</option></select><label for="source-input">Media source</label><input id="source-input" name="source" type="url" required maxlength="4096" placeholder="https://media.example/video.mp4"><button id="create-session" type="submit">Create playback session</button></form><p id="source-status" class="status" role="status"></p></section>
<section aria-labelledby="now-playing-heading"><h2 id="now-playing-heading">Now Playing</h2><dl>
<dt>Item</dt><dd id="item">Not available</dd><dt>State</dt><dd id="playback-state">Not available</dd><dt>Position</dt><dd id="position">Not available</dd>
</dl><div class="controls" aria-label="Playback controls">
<button id="play" type="button" data-command="play">Play</button><button id="pause" type="button" data-command="pause">Pause</button>
<label><span class="sr-only">Seek position in milliseconds</span><input id="seek-position" type="number" min="0" step="1000" value="0"></label><button id="seek" type="button">Seek</button><button id="stop" type="button" data-command="stop">Stop</button>
</div></section>
<section aria-labelledby="display-heading"><h2 id="display-heading">Active Display</h2><dl><dt>Display</dt><dd id="display-id">Not available</dd><dt>Status</dt><dd id="display-status">Not available</dd><dt>Observation</dt><dd id="display-observation">Not available</dd><dt>Error</dt><dd id="display-error">None</dd></dl></section>
<section aria-labelledby="site-heading"><h2 id="site-heading">Site / Account</h2><dl><dt>Site</dt><dd id="site">Unavailable</dd><dt>Account</dt><dd id="account-state">Unknown</dd><dt>Native panel</dt><dd id="panel-status">Not attached</dd></dl></section>
<section aria-labelledby="action-heading"><h2 id="action-heading">Action required</h2><p id="action-required">None</p></section>
<p id="feedback" class="status" role="status" aria-live="polite"></p>
</main>
<script>
(() => {
  const sessionId = new URLSearchParams(location.search).get('session_id');
  const viewEndpoint = sessionId ? `/api/v1/control/${encodeURIComponent(sessionId)}` : null;
  const eventEndpoint = sessionId ? `/api/v1/sessions/${encodeURIComponent(sessionId)}/events` : null;
  let currentView = null, requestInFlight = false, refreshSequence = 0, eventCursor = null, eventPollInFlight = false;
  const $ = id => document.querySelector(id);
  const text = (id, value) => { $(id).textContent = value == null || value === '' ? 'Unavailable' : String(value); };
  const setConnection = (value, error = false) => { text('#connection', value); $('#connection').classList.toggle('error', error); };
  const setFeedback = (value, error = false) => { text('#feedback', value || ''); $('#feedback').classList.toggle('error', error); };
  const boundedCode = payload => payload && typeof payload.code === 'string' ? payload.code : 'COMMAND_REJECTED';
  const recoveryMessage = code => ({
    REVISION_CONFLICT: 'This control view was stale. It was refreshed from Gateway.',
    REQUEST_ID_MISMATCH: 'The command identity was already used differently. The view was refreshed.',
    SESSION_NOT_FOUND: 'This playback session no longer exists.', DISPLAY_OFFLINE: 'The active display is unavailable; playback state remains authoritative.',
    SERVER_UNAVAILABLE: 'The active display reported an unavailable service.', DISPLAY_NOT_FOUND: 'That Display is no longer live; choose another.', DISPLAY_OFFLINE: 'The selected Display is offline; playback state was not created.', SOURCE_NOT_RECOGNIZED: 'The source was not recognized as supported media.', SOURCE_UNSUPPORTED: 'The registered adapter could not prepare that source.', COMMAND_REJECTED: 'Gateway rejected the command; the view was refreshed.'
  })[code] || 'Gateway rejected the request; the view was refreshed.';
  const render = view => {
    currentView = view;
    text('#item', `${view.now_playing.item_id} · revision ${view.now_playing.item_revision}`); text('#playback-state', view.now_playing.state); text('#position', `${view.now_playing.position_ms} ms`);
    text('#display-id', view.active_display.label ? `${view.active_display.label} (${view.active_display.display_id})` : view.active_display.display_id); text('#display-status', view.active_display.online ? 'Online' : 'Offline');
    text('#display-observation', view.active_display.observation || 'Unknown'); text('#display-error', view.active_display.error_code || 'None'); text('#site', view.site.label || view.site.site_id || 'Unavailable');
    text('#account-state', view.site_account_state.state); text('#panel-status', view.native_site_panel.status); text('#action-required', view.action_required ? `${view.action_required.kind}: ${view.action_required.code}` : 'None');
    $('#action-required').classList.toggle('error', Boolean(view.action_required));
    [['#play',view.playback_controls.can_play],['#pause',view.playback_controls.can_pause],['#seek',view.playback_controls.can_seek],['#stop',view.playback_controls.can_stop]].forEach(([id,enabled]) => { $(id).disabled = !enabled || requestInFlight; });
    eventCursor = view.freshness.event_cursor;
  };
  const clearView = () => { currentView = null; ['#item','#playback-state','#position','#display-id','#display-status','#display-observation','#display-error','#site','#account-state','#panel-status','#action-required'].forEach(id => text(id, 'Unavailable')); ['#play','#pause','#seek','#stop'].forEach(id => { $(id).disabled = true; }); };
  const readJson = async response => { try { return await response.json(); } catch (_) { return {}; } };
  const listDisplays = async () => {
    const selector = $('#display-selector');
    if (!selector) return;
    try {
      const response = await fetch('/api/v1/displays', {cache:'no-store'});
      const displays = await readJson(response);
      if (!response.ok || !Array.isArray(displays)) throw new Error('display list unavailable');
      selector.replaceChildren();
      for (const display of displays.filter(item => item && item.online && typeof item.display_id === 'string')) {
        const option = document.createElement('option'); option.value = display.display_id; option.textContent = `${display.label || 'Web Display'} (${display.display_id})`; selector.append(option);
      }
      if (!selector.options.length) { const option = document.createElement('option'); option.value = ''; option.textContent = 'No live Web Displays'; selector.append(option); }
      $('#create-session').disabled = !selector.value;
    } catch (_) { selector.replaceChildren(new Option('Display discovery unavailable', '')); $('#create-session').disabled = true; }
  };
  const createSession = async event => {
    event.preventDefault();
    const source = $('#source-input').value.trim(); const display_id = $('#display-selector').value;
    if (!source || !display_id) { setFeedback('Choose a live Display and enter a media source.', true); return; }
    const request_id = `session-${Date.now()}-${Math.random().toString(36).slice(2,8)}`;
    $('#create-session').disabled = true; $('#source-status').textContent = 'Preparing source through Gateway…';
    try {
      const response = await fetch('/api/v1/sessions', {method:'POST', headers:{'content-type':'application/json'}, body:JSON.stringify({request_id, source, display_id})});
      const payload = await readJson(response);
      if (!response.ok) { const code = boundedCode(payload); $('#source-status').textContent = recoveryMessage(code); $('#source-status').classList.add('error'); await listDisplays(); return; }
      window.location.assign(`/control?session_id=${encodeURIComponent(payload.session_id)}`);
    } catch (_) { $('#source-status').textContent = 'Gateway unavailable; no session was created.'; $('#source-status').classList.add('error'); }
    finally { if (document.body.contains($('#create-session'))) $('#create-session').disabled = false; }
  };
  const refresh = async (message = '') => {
    if (!viewEndpoint) { clearView(); setConnection('Add a bounded session_id query parameter to open a session.', true); return false; }
    const sequence = ++refreshSequence;
    try { const response = await fetch(viewEndpoint, {cache:'no-store'}); const payload = await readJson(response); if (sequence !== refreshSequence) return false;
      if (!response.ok) { clearView(); const code = boundedCode(payload); setConnection(recoveryMessage(code), true); setFeedback(message || recoveryMessage(code), true); return false; }
      render(payload); setConnection('Connected to Gateway.'); if (message) setFeedback(message); return true;
    } catch (_) { if (sequence === refreshSequence) { setConnection('Gateway unavailable; reconnecting…', true); setFeedback('The current view was discarded until Gateway responds.', true); } return false; }
  };
  const command = async (type, extra = {}) => {
    if (!currentView || requestInFlight) return; requestInFlight = true; setFeedback('Sending command…'); render(currentView);
    const request_id = `control-${Date.now()}-${Math.random().toString(36).slice(2,8)}`;
    try { const response = await fetch(`/api/v1/sessions/${encodeURIComponent(sessionId)}/commands`, {method:'POST', headers:{'content-type':'application/json'}, body:JSON.stringify({request_id, expected_session_revision:currentView.freshness.playback.session_revision, command:Object.assign({type},extra)})}); const payload = await readJson(response); const code = boundedCode(payload);
      if (!response.ok) { setFeedback(recoveryMessage(code), true); await refresh(); return; } await refresh('Command accepted; authoritative view refreshed.');
    } catch (_) { setFeedback('Gateway unavailable; the command result is unknown. Resyncing…', true); await refresh(); } finally { requestInFlight = false; if (currentView) render(currentView); }
  };
  const pollEvents = async () => {
    if (!eventEndpoint || eventPollInFlight) return; eventPollInFlight = true;
    try { const after = eventCursor == null ? 0 : eventCursor; const response = await fetch(`${eventEndpoint}?after=${encodeURIComponent(after)}`, {cache:'no-store'}); const payload = await readJson(response);
      if (!response.ok) { await refresh('Event reconnect requested a fresh view.'); return; } if (payload.snapshot_required || Number(payload.cursor) > Number(eventCursor || 0)) await refresh('Gateway event received; authoritative view refreshed.'); setConnection('Connected to Gateway.');
    } catch (_) { setConnection('Event stream disconnected; rebuilding…', true); await refresh(); } finally { eventPollInFlight = false; }
  };
  document.querySelectorAll('[data-command]').forEach(button => button.addEventListener('click', () => command(button.dataset.command)));
  $('#seek').addEventListener('click', () => command('seek', {position_ms:Math.max(0, Number($('#seek-position').value) || 0)}));
  $('#source-form')?.addEventListener('submit', createSession); $('#display-selector')?.addEventListener('change', () => { $('#create-session').disabled = !$('#display-selector').value; });
  window.__controlUi = {refresh, pollEvents, getView:() => currentView, listDisplays}; refresh(); if (!sessionId) listDisplays(); window.setInterval(pollEvents, 1000);
})();
</script></body></html>"##;

async fn control_handler() -> Response {
    Html(CONTROL_PAGE.to_string()).into_response()
}

fn control_view_for_session(
    state: &GatewayState,
    session_id: &str,
) -> Result<ControlView, ControlLookupError> {
    let playback = state.control.snapshot(session_id)?;
    let event_cursor = state.control.events_after(session_id, 0)?.cursor;
    let display = state
        .display_sessions
        .display_view_input(&state.control, &playback.active_display.display_id)
        .unwrap_or_default();
    Ok(ControlView::project(ControlViewInput {
        playback,
        event_cursor: Some(event_cursor),
        site: SiteViewInput::default(),
        browser: BrowserViewInput::default(),
        display,
    }))
}

async fn control_view_handler(
    State(state): State<Arc<GatewayState>>,
    Path(session_id): Path<String>,
) -> Response {
    if !valid_control_selector(&session_id) {
        return control_error_response(
            StatusCode::BAD_REQUEST,
            ControlErrorResponse {
                code: "SESSION_ID_INVALID",
                message: "session selector is invalid",
                current_revision: None,
                transition_id: None,
            },
        );
    }
    match control_view_for_session(&state, &session_id) {
        Ok(view) => Json(view).into_response(),
        Err(ControlLookupError::NotFound) => control_error_response(
            StatusCode::NOT_FOUND,
            ControlErrorResponse {
                code: "SESSION_NOT_FOUND",
                message: "session was not found",
                current_revision: None,
                transition_id: None,
            },
        ),
    }
}

fn valid_control_selector(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
}

#[derive(Clone, Debug, Deserialize)]
struct ControlEventsQuery {
    after: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DisplayHeartbeatRequest {
    lease_token: String,
}

#[derive(Clone, Debug, Deserialize)]
struct DisplayContextQuery {
    session_id: Option<String>,
}

const DISPLAY_LEASE_HEADER: &str = "x-display-lease";

async fn control_session_snapshot_handler(
    State(state): State<Arc<GatewayState>>,
    Path(session_id): Path<String>,
) -> Response {
    match state.control.snapshot(&session_id) {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(ControlLookupError::NotFound) => control_error_response(
            StatusCode::NOT_FOUND,
            ControlErrorResponse {
                code: "SESSION_NOT_FOUND",
                message: "session was not found",
                current_revision: None,
                transition_id: None,
            },
        ),
    }
}

async fn create_session_handler(
    State(state): State<Arc<GatewayState>>,
    request: Result<Json<CreateSessionRequest>, JsonRejection>,
) -> Response {
    let request = match request {
        Ok(Json(request)) => request,
        Err(rejection) => {
            let response = rejection.into_response();
            if response.status() == StatusCode::PAYLOAD_TOO_LARGE {
                return response;
            }
            return (
                StatusCode::BAD_REQUEST,
                Json(CreateSessionErrorResponse {
                    code: "INVALID_JSON",
                    message: "session creation body must be valid structured JSON",
                }),
            )
                .into_response();
        }
    };
    GatewayService { state }
        .create_session(request)
        .into_response()
}

async fn control_session_events_handler(
    State(state): State<Arc<GatewayState>>,
    Path(session_id): Path<String>,
    Query(query): Query<ControlEventsQuery>,
) -> Response {
    match state
        .control
        .events_after(&session_id, query.after.unwrap_or(0))
    {
        Ok(events) => Json(events).into_response(),
        Err(ControlLookupError::NotFound) => control_error_response(
            StatusCode::NOT_FOUND,
            ControlErrorResponse {
                code: "SESSION_NOT_FOUND",
                message: "session was not found",
                current_revision: None,
                transition_id: None,
            },
        ),
    }
}

async fn control_session_command_handler(
    State(state): State<Arc<GatewayState>>,
    Path(session_id): Path<String>,
    request: Result<Json<ControlCommandRequest>, JsonRejection>,
) -> Response {
    let request = match request {
        Ok(Json(request)) => request,
        Err(rejection) => {
            let response = rejection.into_response();
            if response.status() == StatusCode::PAYLOAD_TOO_LARGE {
                return response;
            }
            return control_error_response(
                StatusCode::BAD_REQUEST,
                ControlErrorResponse {
                    code: "INVALID_JSON",
                    message: "command body must be valid structured JSON",
                    current_revision: None,
                    transition_id: None,
                },
            );
        }
    };
    match state.control.execute_command(&session_id, request) {
        Ok(result) => Json(result).into_response(),
        Err(error) => {
            let status = match &error {
                ControlCommandError::NotFound => StatusCode::NOT_FOUND,
                ControlCommandError::Validation(_) => StatusCode::BAD_REQUEST,
                ControlCommandError::Playback(_) => StatusCode::CONFLICT,
            };
            control_error_response(status, error.response())
        }
    }
}

fn control_error_response(status: StatusCode, error: ControlErrorResponse) -> Response {
    (status, Json(error)).into_response()
}

async fn display_register_handler(
    State(state): State<Arc<GatewayState>>,
    request: Result<Json<DisplayRegistration>, JsonRejection>,
) -> Response {
    let input = match request {
        Ok(Json(input)) => input,
        Err(rejection) => return display_json_rejection(rejection),
    };
    match state.display_sessions.register(&state.control, input) {
        Ok(response) => Json(response).into_response(),
        Err(error) => display_session_error_response(error),
    }
}

async fn live_displays_handler(
    State(state): State<Arc<GatewayState>>,
) -> Json<Vec<LiveDisplayView>> {
    Json(state.display_sessions.live_displays())
}

async fn display_heartbeat_handler(
    State(state): State<Arc<GatewayState>>,
    Path(display_id): Path<String>,
    request: Result<Json<DisplayHeartbeatRequest>, JsonRejection>,
) -> Response {
    let input = match request {
        Ok(Json(input)) => input,
        Err(rejection) => return display_json_rejection(rejection),
    };
    match state
        .display_sessions
        .heartbeat(&display_id, &input.lease_token)
    {
        Ok(response) => Json(response).into_response(),
        Err(error) => display_session_error_response(error),
    }
}

async fn display_context_handler(
    State(state): State<Arc<GatewayState>>,
    Path(display_id): Path<String>,
    Query(query): Query<DisplayContextQuery>,
    headers: HeaderMap,
) -> Response {
    let Some(lease_token) = headers
        .get(DISPLAY_LEASE_HEADER)
        .and_then(|value| value.to_str().ok())
    else {
        return display_session_error_response(DisplaySessionError::LeaseInvalid);
    };
    match state.display_sessions.context_for_session(
        &state.control,
        &display_id,
        lease_token,
        query.session_id.as_deref(),
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => display_session_error_response(error),
    }
}

async fn display_rendering_handler(
    State(state): State<Arc<GatewayState>>,
    Path(display_id): Path<String>,
    Query(query): Query<DisplayContextQuery>,
    headers: HeaderMap,
) -> Response {
    let Some(lease_token) = headers
        .get(DISPLAY_LEASE_HEADER)
        .and_then(|value| value.to_str().ok())
    else {
        return display_session_error_response(DisplaySessionError::LeaseInvalid);
    };

    let context = match query.session_id.as_deref() {
        Some(session_id) => state.display_sessions.context_for_session(
            &state.control,
            &display_id,
            lease_token,
            Some(session_id),
        ),
        None => state.display_sessions.context_for_active_display(
            &state.control,
            &display_id,
            lease_token,
        ),
    };
    let context = match context {
        Ok(context) => context,
        Err(error) => return display_session_error_response(error),
    };
    if !context.is_current_display {
        return display_session_error_response(DisplaySessionError::StaleContext);
    }
    let Ok(snapshot) = state.control.snapshot(&context.session_id) else {
        return display_session_error_response(DisplaySessionError::SessionNotFound);
    };
    let Some(media) = state.source_sessions.media_for_snapshot(&snapshot) else {
        return display_session_error_response(DisplaySessionError::SessionNotFound);
    };
    Json(DisplayRenderingResponse { context, media }).into_response()
}

async fn display_callback_handler(
    State(state): State<Arc<GatewayState>>,
    Path(display_id): Path<String>,
    request: Result<Json<DisplayCallback>, JsonRejection>,
) -> Response {
    let Json(callback) = match request {
        Ok(value) => value,
        Err(rejection) => return display_json_rejection(rejection),
    };
    if callback.display_id != display_id {
        return display_session_error_response(DisplaySessionError::StaleContext);
    }
    // The callback service validates the lease and all R007 context before
    // accepting any telemetry or generic status/error observation.
    match state.display_sessions.callback(&state.control, callback) {
        Ok(response) => Json(response).into_response(),
        Err(error) => display_session_error_response(error),
    }
}

fn display_json_rejection(rejection: JsonRejection) -> Response {
    let response = rejection.into_response();
    if response.status() == StatusCode::PAYLOAD_TOO_LARGE {
        response
    } else {
        display_session_error_response(DisplaySessionError::InvalidCallback)
    }
}

fn display_session_error_response(error: DisplaySessionError) -> Response {
    let status = match error {
        DisplaySessionError::SessionNotFound
        | DisplaySessionError::SessionNotAttached
        | DisplaySessionError::RegistrationNotFound => StatusCode::NOT_FOUND,
        DisplaySessionError::LeaseInvalid | DisplaySessionError::LeaseExpired => {
            StatusCode::UNAUTHORIZED
        }
        DisplaySessionError::AlreadyRegistered | DisplaySessionError::StaleContext => {
            StatusCode::CONFLICT
        }
        DisplaySessionError::StaleTelemetry => StatusCode::CONFLICT,
        DisplaySessionError::InvalidIdentifier(_)
        | DisplaySessionError::InvalidLabel
        | DisplaySessionError::InvalidCapabilities
        | DisplaySessionError::InvalidCallback => StatusCode::BAD_REQUEST,
    };
    (status, Json(DisplaySessionErrorResponse::from(&error))).into_response()
}

async fn probe_state_handler(State(state): State<Arc<GatewayState>>) -> Json<ProbeStateSnapshot> {
    Json(state.probe.state_snapshot())
}

async fn probe_events_handler(
    State(state): State<Arc<GatewayState>>,
    Query(query): Query<ProbeEventsQuery>,
) -> Json<ProbeEventsResponse> {
    Json(state.probe.events_after(query.after.unwrap_or(0)))
}

async fn probe_command_handler(
    State(state): State<Arc<GatewayState>>,
    Json(input): Json<PlayCommandRequest>,
) -> Response {
    let request_id = input
        .request_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
    if request_id.len() > 128 {
        return (StatusCode::BAD_REQUEST, "request_id_too_long").into_response();
    }
    if !request_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
    {
        return (StatusCode::BAD_REQUEST, "request_id_invalid").into_response();
    }
    let (command, duplicate) = state.probe.issue_play(request_id);
    Json(PlayCommandResponse {
        accepted: true,
        duplicate,
        command,
    })
    .into_response()
}

async fn probe_telemetry_handler(
    State(state): State<Arc<GatewayState>>,
    Json(input): Json<ProbeTelemetryRequest>,
) -> Json<serde_json::Value> {
    let sequence = state.probe.append_telemetry(input);
    Json(serde_json::json!({ "accepted": true, "sequence": sequence }))
}

async fn probe_reset_handler(State(state): State<Arc<GatewayState>>) -> Json<serde_json::Value> {
    state.probe.reset();
    Json(serde_json::json!({ "reset": true }))
}

async fn fixture_handler(
    State(state): State<Arc<GatewayState>>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let authorized = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == "Bearer r001-fixture-secret");
    if !authorized {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let path = state
        .fixture_mp4
        .read()
        .expect("fixture path poisoned")
        .clone();
    let Some(path) = path else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(bytes) = tokio::fs::read(path).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    ranged_bytes_response(method, &headers, bytes)
}

async fn subtitle_fixture_handler(
    State(state): State<Arc<GatewayState>>,
    method: Method,
) -> Response {
    let path = state
        .fixture_vtt
        .read()
        .expect("subtitle fixture path poisoned")
        .clone();
    let Some(path) = path else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(bytes) = tokio::fs::read(path).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/vtt; charset=utf-8")
        .header(CONTENT_LENGTH, bytes.len().to_string())
        .body(if method == Method::HEAD {
            Body::empty()
        } else {
            Body::from(bytes)
        })
        .expect("subtitle fixture response")
}

fn ranged_bytes_response(method: Method, request_headers: &HeaderMap, bytes: Vec<u8>) -> Response {
    let len = bytes.len();
    let mut status = StatusCode::OK;
    let mut body = bytes;
    let mut content_range = None;
    if let Some(range) = request_headers
        .get(RANGE)
        .and_then(|value| value.to_str().ok())
    {
        if let Some((start, end)) = parse_single_range(range, len) {
            status = StatusCode::PARTIAL_CONTENT;
            content_range = Some(format!("bytes {start}-{end}/{len}"));
            body = body[start..=end].to_vec();
        } else {
            return StatusCode::RANGE_NOT_SATISFIABLE.into_response();
        }
    }
    let body_len = body.len();
    let mut builder = Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "video/mp4")
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, body_len.to_string());
    if let Some(value) = content_range {
        builder = builder.header(CONTENT_RANGE, value);
    }
    builder
        .body(if method == Method::HEAD {
            Body::empty()
        } else {
            Body::from(body)
        })
        .expect("fixture response")
}

fn parse_single_range(value: &str, len: usize) -> Option<(usize, usize)> {
    let range = value.strip_prefix("bytes=")?;
    if range.contains(',') {
        return None;
    }
    let (start, end) = range.split_once('-')?;
    let start: usize = start.parse().ok()?;
    if start >= len {
        return None;
    }
    let end = if end.is_empty() {
        len - 1
    } else {
        end.parse::<usize>().ok()?.min(len - 1)
    };
    (start <= end).then_some((start, end))
}

async fn stream_handler(
    State(state): State<Arc<GatewayState>>,
    Path((token, session, item, revision, resource)): Path<(String, String, String, u64, String)>,
    method: Method,
    request_headers: HeaderMap,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let binding = Binding::new(&session, &item, revision, &resource);
    let record = match state.store.get(&token, &binding) {
        Ok(record) => record,
        Err(CapabilityError::NotFound) => {
            return (StatusCode::NOT_FOUND, "INVALID_MEDIA_CAPABILITY").into_response();
        }
        Err(CapabilityError::Expired) => {
            return (StatusCode::GONE, "EXPIRED_MEDIA_CAPABILITY").into_response();
        }
        Err(CapabilityError::BindingMismatch) => {
            return (StatusCode::FORBIDDEN, "MEDIA_CAPABILITY_BINDING_MISMATCH").into_response();
        }
    };

    let (upstream, final_url) =
        match fetch_upstream(&state, &record, &method, &request_headers).await {
            Ok(value) => value,
            Err(code) => return (StatusCode::BAD_GATEWAY, code).into_response(),
        };

    if request_headers.contains_key(RANGE) && upstream.status() == StatusCode::OK {
        return (StatusCode::BAD_GATEWAY, "UPSTREAM_RANGE_UNSUPPORTED").into_response();
    }

    let status = upstream.status();
    let response_headers = filtered_response_headers(upstream.headers());
    if !status.is_success() {
        return response_from_parts(status, response_headers, Body::empty());
    }
    if method == Method::HEAD {
        return response_from_parts(status, response_headers, Body::empty());
    }

    if record.resource.protocol == StreamProtocol::Hls
        && is_manifest(&final_url, upstream.headers())
    {
        let bytes = match collect_limited(upstream, MAX_MANIFEST_BYTES).await {
            Ok(bytes) => bytes,
            Err(code) => return (StatusCode::BAD_GATEWAY, code).into_response(),
        };
        let manifest = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => return (StatusCode::BAD_GATEWAY, "INVALID_HLS_MANIFEST").into_response(),
        };
        let rewritten = match rewrite_manifest(&state, &record, &final_url, &manifest).await {
            Ok(text) => text,
            Err(code) => return (StatusCode::BAD_GATEWAY, code).into_response(),
        };
        let mut headers = response_headers;
        headers.remove(CONTENT_LENGTH);
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.apple.mpegurl"),
        );
        return response_from_parts(status, headers, Body::from(rewritten));
    }

    let guard = ActiveGuard::new(state.active_streams.clone());
    let upstream_stream = Box::pin(upstream.bytes_stream());
    let guarded = stream::unfold((upstream_stream, guard), |(mut source, guard)| async move {
        match source.next().await {
            Some(Ok(chunk)) => Some((Ok::<Bytes, reqwest::Error>(chunk), (source, guard))),
            Some(Err(error)) => Some((Err(error), (source, guard))),
            None => None,
        }
    });
    response_from_parts(status, response_headers, Body::from_stream(guarded))
}

async fn fetch_upstream(
    state: &GatewayState,
    record: &CapabilityRecord,
    method: &Method,
    request_headers: &HeaderMap,
) -> Result<(reqwest::Response, Url), &'static str> {
    let mut url = record.resource.url.clone();
    for hop in 0..=MAX_REDIRECTS {
        let egress_policy = state
            .egress_policy
            .read()
            .expect("egress policy poisoned")
            .clone();
        let validated_target = egress_policy
            .validate_and_resolve(&url, &record.resource.egress_scope)
            .await
            .map_err(egress_error_code)?;
        let client = validated_target
            .pinned_client()
            .map_err(|_| "UPSTREAM_CLIENT_FAILED")?;
        let mut request = client.request(method.clone(), url.clone());
        request = request.headers(record.resource.public_headers.clone());
        request = request.headers(record.resource.secret_headers.clone());
        if let Some(value) = request_headers.get(RANGE) {
            request = request.header(RANGE, value);
        }
        if let Some(value) = request_headers.get(IF_RANGE) {
            request = request.header(IF_RANGE, value);
        }
        let response = request
            .send()
            .await
            .map_err(|_| "UPSTREAM_REQUEST_FAILED")?;
        if response.status().is_redirection() {
            if hop == MAX_REDIRECTS {
                return Err("TOO_MANY_REDIRECTS");
            }
            let location = response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .ok_or("INVALID_REDIRECT")?;
            url = url.join(location).map_err(|_| "INVALID_REDIRECT")?;
            continue;
        }
        return Ok((response, url));
    }
    Err("TOO_MANY_REDIRECTS")
}

fn egress_error_code(error: EgressPolicyError) -> &'static str {
    match error {
        EgressPolicyError::InvalidScheme => "EGRESS_SCHEME_REJECTED",
        EgressPolicyError::MissingHost => "EGRESS_HOST_REJECTED",
        EgressPolicyError::InvalidPort => "EGRESS_PORT_REJECTED",
        EgressPolicyError::DnsLookupFailed => "EGRESS_DNS_FAILED",
        EgressPolicyError::TargetRejected
        | EgressPolicyError::LocalServiceOriginMismatch
        | EgressPolicyError::LocalServiceNotConfigured => "EGRESS_TARGET_REJECTED",
    }
}

fn is_manifest(url: &Url, headers: &HeaderMap) -> bool {
    if url.path().to_ascii_lowercase().ends_with(".m3u8") {
        return true;
    }
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("mpegurl") || value.contains("m3u"))
}

async fn collect_limited(
    response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, &'static str> {
    if response
        .content_length()
        .is_some_and(|length| length as usize > limit)
    {
        return Err("HLS_MANIFEST_TOO_LARGE");
    }
    let mut output = Vec::new();
    let mut source = response.bytes_stream();
    while let Some(chunk) = source.next().await {
        let chunk = chunk.map_err(|_| "HLS_MANIFEST_INTERRUPTED")?;
        if output.len() + chunk.len() > limit {
            return Err("HLS_MANIFEST_TOO_LARGE");
        }
        output.extend_from_slice(&chunk);
    }
    Ok(output)
}

async fn rewrite_manifest(
    state: &GatewayState,
    parent: &CapabilityRecord,
    base: &Url,
    manifest: &str,
) -> Result<String, &'static str> {
    let mut output = String::with_capacity(manifest.len() + 128);
    for line in manifest.lines() {
        let rewritten = if line.trim().is_empty() {
            String::new()
        } else if line.starts_with('#') {
            rewrite_uri_attribute(state, parent, base, line).await?
        } else {
            issue_hls_child(state, parent, base, line.trim()).await?
        };
        output.push_str(&rewritten);
        output.push('\n');
    }
    Ok(output)
}

async fn rewrite_uri_attribute(
    state: &GatewayState,
    parent: &CapabilityRecord,
    base: &Url,
    line: &str,
) -> Result<String, &'static str> {
    let Some(start) = line.find("URI=\"") else {
        return Ok(line.to_string());
    };
    let value_start = start + 5;
    let rest = &line[value_start..];
    let Some(end_relative) = rest.find('"') else {
        return Err("INVALID_HLS_URI_ATTRIBUTE");
    };
    let value_end = value_start + end_relative;
    let rewritten = issue_hls_child(state, parent, base, &line[value_start..value_end]).await?;
    Ok(format!(
        "{}{}{}",
        &line[..value_start],
        rewritten,
        &line[value_end..]
    ))
}

async fn issue_hls_child(
    state: &GatewayState,
    parent: &CapabilityRecord,
    base: &Url,
    target: &str,
) -> Result<String, &'static str> {
    let url = base.join(target).map_err(|_| "INVALID_HLS_CHILD_URI")?;
    let egress_policy = state
        .egress_policy
        .read()
        .expect("egress policy poisoned")
        .clone();
    egress_policy
        .validate(&url, &parent.resource.egress_scope)
        .await
        .map_err(egress_error_code)?;
    let protocol = if url.path().to_ascii_lowercase().ends_with(".m3u8") {
        StreamProtocol::Hls
    } else {
        StreamProtocol::HttpFile
    };
    let child_id = format!(
        "hls-{}",
        state.store.sequence.fetch_add(1, Ordering::Relaxed)
    );
    let binding = Binding::new(
        &parent.binding.session_id,
        &parent.binding.item_id,
        parent.binding.item_revision,
        &child_id,
    );
    let ttl = parent.expires_at.saturating_duration_since(Instant::now());
    if ttl.is_zero() {
        return Err("EXPIRED_MEDIA_CAPABILITY");
    }
    let token = state.store.issue(
        binding.clone(),
        UpstreamResource {
            url,
            protocol,
            public_headers: parent.resource.public_headers.clone(),
            secret_headers: parent.resource.secret_headers.clone(),
            egress_scope: parent.resource.egress_scope.clone(),
        },
        ttl,
    );
    Ok(stream_path(&token, &binding))
}

fn filtered_response_headers(headers: &HeaderMap) -> HeaderMap {
    let mut output = HeaderMap::new();
    for name in [
        CONTENT_TYPE,
        CONTENT_LENGTH,
        CONTENT_RANGE,
        ACCEPT_RANGES,
        ETAG,
        LAST_MODIFIED,
        CACHE_CONTROL,
    ] {
        if let Some(value) = headers.get(&name) {
            output.insert(name, value.clone());
        }
    }
    output
}

fn response_from_parts(status: StatusCode, headers: HeaderMap, body: Body) -> Response {
    let mut response = Response::builder()
        .status(status)
        .body(body)
        .expect("response");
    *response.headers_mut() = headers;
    response
}

struct ActiveGuard(Arc<AtomicUsize>);

impl ActiveGuard {
    fn new(counter: Arc<AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::SeqCst);
        Self(counter)
    }
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn resolved_secret_headers_are_rejected() {
        let stream = ResolvedStream {
            id: "x".into(),
            protocol: StreamProtocol::HttpFile,
            url: Url::parse("https://example.com/a.mp4").unwrap(),
            public_headers: BTreeMap::from([("Authorization".into(), "Bearer leak".into())]),
            upstream_access_ref: None,
        };
        assert!(matches!(
            GatewayService::resource_from_resolved(&stream, EgressScope::PublicWeb),
            Err(GatewayError::SecretHeader)
        ));
    }

    #[test]
    fn bearer_like_values_and_secret_header_names_are_rejected() {
        for header in [
            ("x-trace", "Bearer hidden-secret"),
            ("Cookie", "session=hidden-secret"),
            ("X-Api-Key", "hidden-secret"),
        ] {
            let stream = ResolvedStream {
                id: "x".into(),
                protocol: StreamProtocol::HttpFile,
                url: Url::parse("https://example.com/a.mp4").unwrap(),
                public_headers: BTreeMap::from([(header.0.into(), header.1.into())]),
                upstream_access_ref: None,
            };
            assert!(matches!(
                GatewayService::resource_from_resolved(&stream, EgressScope::PublicWeb),
                Err(GatewayError::SecretHeader)
            ));
        }
    }

    #[test]
    fn subtitle_view_issues_only_an_opaque_gateway_path() {
        let service = GatewayService::new(8);
        let subtitle = ResolvedSubtitle {
            id: "english".into(),
            url: Url::parse("https://example.test/captions.vtt").unwrap(),
            content_type: "text/vtt".into(),
            language: Some("en".into()),
            label: Some("English".into()),
            public_headers: BTreeMap::new(),
            upstream_access_ref: Some("server-only-ref".into()),
        };
        let view = service
            .subtitle_track_view(
                &subtitle,
                Binding::new("session", "item", 1, "english"),
                EgressScope::PublicWeb,
                Duration::from_secs(30),
            )
            .unwrap();
        assert_eq!(view.format, "webvtt");
        assert!(view.gateway_path.starts_with("/stream/"));
        assert!(!view.gateway_path.contains("example.test"));
        assert!(!view.gateway_path.contains("server-only-ref"));
    }

    #[test]
    fn subtitle_view_rejects_unsupported_content_and_secret_headers() {
        let service = GatewayService::new(8);
        let mut subtitle = ResolvedSubtitle {
            id: "bad".into(),
            url: Url::parse("https://example.test/captions.srt").unwrap(),
            content_type: "text/srt".into(),
            language: None,
            label: None,
            public_headers: BTreeMap::new(),
            upstream_access_ref: None,
        };
        assert!(matches!(
            service.subtitle_track_view(
                &subtitle,
                Binding::new("session", "item", 1, "bad"),
                EgressScope::PublicWeb,
                Duration::from_secs(30),
            ),
            Err(GatewayError::UnsupportedSubtitleContentType)
        ));
        subtitle.content_type = "text/vtt".into();
        subtitle
            .public_headers
            .insert("Authorization".into(), "Bearer fixture-secret".into());
        assert!(matches!(
            service.subtitle_track_view(
                &subtitle,
                Binding::new("session", "item", 1, "bad"),
                EgressScope::PublicWeb,
                Duration::from_secs(30),
            ),
            Err(GatewayError::SecretHeader)
        ));
    }

    #[test]
    fn single_range_parser_rejects_multi_range() {
        assert_eq!(parse_single_range("bytes=2-5", 10), Some((2, 5)));
        assert_eq!(parse_single_range("bytes=2-", 10), Some((2, 9)));
        assert_eq!(parse_single_range("bytes=0-1,4-5", 10), None);
    }

    #[test]
    fn probe_request_id_is_idempotent_and_cursor_is_replayable() {
        let probe = ProbeStore::default();
        let (first, duplicate) = probe.issue_play("case-a-1".into());
        assert!(!duplicate);
        let (second, duplicate) = probe.issue_play("case-a-1".into());
        assert!(duplicate);
        assert_eq!(first.sequence, second.sequence);
        assert_eq!(probe.events_after(0).events.len(), 1);
        assert_eq!(probe.events_after(first.sequence).events.len(), 0);
    }

    #[test]
    fn probe_telemetry_is_bounded_and_redacts_header_like_text() {
        let probe = ProbeStore::default();
        let sequence = probe.append_telemetry(ProbeTelemetryRequest {
            kind: "play_attempt".into(),
            command_id: Some("case-a-1".into()),
            attempt_id: Some("attempt-1".into()),
            result: Some("reject".into()),
            error_name: Some("NotAllowedError".into()),
            error_message: Some("Cookie Bearer secret".into()),
            muted: Some(false),
            volume: Some(1.0),
            detail: None,
        });
        assert_eq!(sequence, 1);
        let snapshot = probe.state_snapshot();
        assert_eq!(snapshot.telemetry.len(), 1);
        assert!(
            !snapshot.telemetry[0]
                .error_message
                .as_deref()
                .unwrap()
                .contains("Bearer secret")
        );
    }
}
pub mod playback;
