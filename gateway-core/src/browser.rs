//! Target-neutral contracts for the Site Browser Worker and Native Site Panel.
//!
//! This module deliberately contains no Chromium, Playwright, site DOM, or
//! account interpretation.  A worker only reports generic browser facts.  A
//! Site Plugin remains responsible for interpreting those facts into a
//! `SourceLocator`, `AccountState`, or `NativePanelState`.

use crate::{EgressPolicy, EgressScope};
use futures_util::{FutureExt, future::BoxFuture};
use site_adapter_api::{
    AdapterError, BrowserMediaCandidate, BrowserObservation, ResolveContext, ResolvedMedia,
    ServerOwnedObservation, SiteAdapterRegistry, validate_browser_observation,
    validate_server_owned_observation,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use url::Url;
use uuid::Uuid;

pub const BROWSER_EVENT_VERSION: u16 = 1;
pub const MAX_BROWSER_EVENTS: usize = 200;
pub const MAX_BROWSER_RESOURCES: usize = 64;
pub const MAX_BROWSER_CANDIDATES: usize = 16;
pub const BROWSER_HANDOFF_TTL: Duration = Duration::from_secs(60);

pub type BrowserFuture<'a, T> = BoxFuture<'a, Result<T, BrowserError>>;

/// Stable, generic failure semantics.  Variants intentionally carry no
/// arbitrary detail so they are safe to expose in ordinary diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserError {
    WorkerUnavailable,
    ProfileAttachFailed,
    NavigationDenied,
    WorkerCrashed,
    WorkerTimeout,
    SessionExpired,
    PanelDisconnected,
    InterpretationUnavailable,
    SessionClosed,
    OperationCancelled,
    InvalidSession,
    InvalidInput,
    CandidateCaptureUnavailable,
    CandidateMaterialInvalid,
    ResourceLimitExceeded,
}

impl BrowserError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::WorkerUnavailable => "WORKER_UNAVAILABLE",
            Self::ProfileAttachFailed => "PROFILE_ATTACH_FAILED",
            Self::NavigationDenied => "NAVIGATION_DENIED",
            Self::WorkerCrashed => "WORKER_CRASHED",
            Self::WorkerTimeout => "WORKER_TIMEOUT",
            Self::SessionExpired => "SESSION_EXPIRED",
            Self::PanelDisconnected => "PANEL_DISCONNECTED",
            Self::InterpretationUnavailable => "INTERPRETATION_UNAVAILABLE",
            Self::SessionClosed => "SESSION_CLOSED",
            Self::OperationCancelled => "OPERATION_CANCELLED",
            Self::InvalidSession => "INVALID_SESSION",
            Self::InvalidInput => "INVALID_INPUT",
            Self::CandidateCaptureUnavailable => "CANDIDATE_CAPTURE_UNAVAILABLE",
            Self::CandidateMaterialInvalid => "CANDIDATE_MATERIAL_INVALID",
            Self::ResourceLimitExceeded => "RESOURCE_LIMIT_EXCEEDED",
        }
    }
}

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for BrowserError {}

/// Opaque worker/session identity.  It is not a Vault path or a site
/// identifier.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct BrowserSessionId(String);

impl BrowserSessionId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4().simple().to_string())
    }
}

impl fmt::Debug for BrowserSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BrowserSessionId")
            .field(&"[opaque]")
            .finish()
    }
}

/// An opaque, short-lived reference to profile material owned by the Vault.
/// The reference contains neither a filesystem path nor profile contents.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProfileAttachmentRef(String);

impl ProfileAttachmentRef {
    /// Only trusted Core/Vault infrastructure may issue production refs.  A
    /// plugin or browser-facing caller cannot construct this type.
    #[allow(dead_code)]
    pub(crate) fn from_vault_issued(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    pub(crate) fn token(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ProfileAttachmentRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ProfileAttachmentRef")
            .field(&"[opaque]")
            .finish()
    }
}

/// Material returned only across the crate-private Browser Worker/Vault seam.
/// It is intentionally not serializable and its debug output contains no
/// session values or profile bytes.
#[derive(Clone, Eq, PartialEq)]
pub struct BrowserCandidateMaterial {
    pub(crate) cookie_header: Option<String>,
    pub(crate) authorization_header: Option<String>,
    pub(crate) local_storage: Option<String>,
    pub(crate) browser_profile: Option<Vec<u8>>,
}

impl BrowserCandidateMaterial {
    pub(crate) fn from_profile(profile: Vec<u8>) -> Result<Self, BrowserError> {
        if profile.is_empty() || profile.len() > MAX_CANDIDATE_PROFILE_BYTES {
            return Err(BrowserError::CandidateMaterialInvalid);
        }
        Ok(Self {
            cookie_header: None,
            authorization_header: None,
            local_storage: None,
            browser_profile: Some(profile),
        })
    }

    #[cfg(test)]
    pub(crate) fn fixture(label: &str) -> Self {
        Self {
            cookie_header: Some(format!("cookie-{label}")),
            authorization_header: Some(format!("Bearer authorization-{label}")),
            local_storage: Some(format!("local-storage-{label}")),
            browser_profile: Some(format!("profile-{label}").into_bytes()),
        }
    }
}

impl fmt::Debug for BrowserCandidateMaterial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserCandidateMaterial")
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
                &self.browser_profile.as_ref().map(Vec::len),
            )
            .finish()
    }
}

pub(crate) const MAX_CANDIDATE_PROFILE_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_CANDIDATE_SECRET_TEXT_BYTES: usize = 128 * 1024;
pub(crate) const MAX_CANDIDATE_PROFILE_FILES: usize = 256;

/// A generic worker mode.  Account/login success is interpreted by a Site
/// Plugin, never by this contract or the fake worker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserAuthMode {
    Passive,
    Interactive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserSession {
    id: BrowserSessionId,
    mode: BrowserAuthMode,
}

impl BrowserSession {
    pub(crate) fn new_for_runtime(mode: BrowserAuthMode) -> Self {
        Self {
            id: BrowserSessionId::new(),
            mode,
        }
    }

    pub fn id(&self) -> &BrowserSessionId {
        &self.id
    }

    pub fn mode(&self) -> BrowserAuthMode {
        self.mode
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserStatus {
    Open,
    Closed,
    Crashed,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BrowserOperationId(u64);

impl BrowserOperationId {
    pub const fn value(self) -> u64 {
        self.0
    }

    pub const fn from_value(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct BrowserNavigationRequest {
    operation_id: BrowserOperationId,
    url: Url,
}

/// A resource fact that is safe to cross the worker event boundary.  The
/// corresponding URL and response headers remain in the server-owned media
/// ledger and are never part of this value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserResourceObservation {
    pub operation_id: BrowserOperationId,
    pub candidate: BrowserMediaCandidate,
}

/// The only worker-to-plugin acquisition boundary.  `server_observation` is
/// deliberately non-serializable at this layer and contains the short-lived
/// server-side media references needed by the adapter.
#[derive(Clone, Eq, PartialEq)]
pub struct BrowserObservationPayload {
    pub operation_id: BrowserOperationId,
    pub observation: BrowserObservation,
    pub server_observation: ServerOwnedObservation,
}

impl fmt::Debug for BrowserObservationPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserObservationPayload")
            .field("operation_id", &self.operation_id)
            .field("observation", &self.observation)
            .field("server_observation", &"[server-owned]")
            .finish()
    }
}

/// A plugin-facing observation bound by Core without interpreting the
/// locator payload.  The handoff is one-shot at the worker boundary and has a
/// finite lifetime; callers must still use the normal registry/API boundary
/// to resolve the opaque locator.
#[derive(Clone, Eq, PartialEq)]
pub struct BrowserObservationHandoff {
    session_id: BrowserSessionId,
    operation_id: BrowserOperationId,
    locator: site_adapter_api::SourceLocator,
    observation: BrowserObservation,
    server_observation: ServerOwnedObservation,
    expires_at: Instant,
}

impl fmt::Debug for BrowserObservationHandoff {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserObservationHandoff")
            .field("session_id", &self.session_id)
            .field("operation_id", &self.operation_id)
            .field("locator", &"[opaque]")
            .field("observation", &self.observation)
            .field("server_observation", &"[server-owned]")
            .field("expires_at", &"[short-lived]")
            .finish()
    }
}

impl BrowserObservationHandoff {
    /// Collect a single worker payload and bind it to the caller-provided
    /// opaque locator.  Core compares identity only; the owning plugin
    /// remains responsible for interpreting the locator and selecting a
    /// candidate during `resolve`.
    pub fn take_from_worker(
        worker: &dyn BrowserWorker,
        session_id: &BrowserSessionId,
        operation_id: BrowserOperationId,
        locator: site_adapter_api::SourceLocator,
        ttl: Duration,
    ) -> Result<Option<Self>, BrowserError> {
        let Some(payload) = worker.take_observation(session_id, operation_id)? else {
            return Ok(None);
        };
        Self::bind(session_id.clone(), operation_id, locator, payload, ttl).map(Some)
    }

    pub fn bind(
        session_id: BrowserSessionId,
        operation_id: BrowserOperationId,
        locator: site_adapter_api::SourceLocator,
        payload: BrowserObservationPayload,
        ttl: Duration,
    ) -> Result<Self, BrowserError> {
        if payload.operation_id != operation_id
            || validate_browser_observation(&payload.observation).is_err()
            || validate_server_owned_observation(&payload.server_observation).is_err()
            || payload.observation.observation_id != payload.server_observation.observation_id
        {
            return Err(BrowserError::InvalidInput);
        }
        Ok(Self {
            session_id,
            operation_id,
            locator,
            observation: payload.observation,
            server_observation: payload.server_observation,
            expires_at: Instant::now() + ttl,
        })
    }

    pub fn session_id(&self) -> &BrowserSessionId {
        &self.session_id
    }

    pub fn operation_id(&self) -> BrowserOperationId {
        self.operation_id
    }

    pub fn locator(&self) -> &site_adapter_api::SourceLocator {
        &self.locator
    }

    pub fn observation(&self) -> &BrowserObservation {
        &self.observation
    }

    pub fn server_observation(&self) -> &ServerOwnedObservation {
        &self.server_observation
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= Instant::now()
    }

    pub fn resolve(&self, registry: &SiteAdapterRegistry) -> Result<ResolvedMedia, AdapterError> {
        if self.is_expired() {
            return Err(AdapterError::ObservationExpired);
        }
        registry.resolve_with_context(
            &self.locator,
            ResolveContext {
                browser_observation: Some(&self.observation),
                server_observation: Some(&self.server_observation),
                authenticated_session: None,
            },
        )
    }
}

impl fmt::Debug for BrowserNavigationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserNavigationRequest")
            .field("operation_id", &self.operation_id)
            .field("url", &redacted_url(&self.url))
            .finish()
    }
}

impl BrowserNavigationRequest {
    pub fn new(url: Url) -> Self {
        Self {
            operation_id: BrowserOperationId(Uuid::new_v4().as_u128() as u64),
            url,
        }
    }

    pub fn with_operation_id(operation_id: BrowserOperationId, url: Url) -> Self {
        Self { operation_id, url }
    }

    pub fn operation_id(&self) -> BrowserOperationId {
        self.operation_id
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}

#[derive(Clone)]
pub struct R008NavigationPolicy {
    egress: Arc<RwLock<EgressPolicy>>,
    scope: EgressScope,
}

impl R008NavigationPolicy {
    /// The public browser-facing context is always public-web scoped.  A
    /// caller cannot choose a configured local service through this API.
    pub fn public_web(egress: EgressPolicy) -> Self {
        Self {
            egress: Arc::new(RwLock::new(egress)),
            scope: EgressScope::PublicWeb,
        }
    }

    /// Trusted Core-only path for a configured internal integration.  It is
    /// intentionally unavailable to plugins and browser-facing callers.
    #[allow(dead_code)]
    pub(crate) fn configured_local_service(
        egress: EgressPolicy,
        service_name: impl Into<String>,
    ) -> Self {
        Self {
            egress: Arc::new(RwLock::new(egress)),
            scope: EgressScope::ConfiguredLocalService(service_name.into()),
        }
    }

    fn authorize<'a>(&'a self, request: &'a BrowserNavigationRequest) -> BrowserFuture<'a, ()> {
        Box::pin(async move { self.authorize_url(request.url()).await })
    }

    pub(crate) async fn authorize_url(&self, url: &Url) -> Result<(), BrowserError> {
        let policy = self
            .egress
            .read()
            .map_err(|_| BrowserError::WorkerUnavailable)?
            .clone();
        policy
            .validate(url, &self.scope)
            .await
            .map_err(|_| BrowserError::NavigationDenied)
    }
}

fn redacted_url(url: &Url) -> String {
    let mut safe = url.clone();
    safe.set_query(None);
    safe.set_fragment(None);
    safe.to_string()
}

pub(crate) fn redacted_event_url(url: &Url) -> Url {
    let mut safe = url.clone();
    safe.set_username("").ok();
    safe.set_password(None).ok();
    safe.set_fragment(None);
    let query = safe
        .query_pairs()
        .filter(|(name, value)| !sensitive_url_component(name) && !sensitive_url_component(value))
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();
    safe.set_query(None);
    for (name, value) in query {
        safe.query_pairs_mut().append_pair(&name, &value);
    }
    safe
}

fn sensitive_url_component(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "authorization",
        "bearer",
        "cookie",
        "password",
        "secret",
        "session",
        "signature",
        "signed",
        "token",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

pub(crate) fn redacted_title(title: &str) -> String {
    if title.chars().any(char::is_control) || sensitive_url_component(title) {
        return "[REDACTED]".into();
    }
    title.chars().take(512).collect()
}

#[derive(Clone, Eq, PartialEq)]
pub enum BrowserInput {
    Key {
        key: String,
    },
    Pointer {
        x: i32,
        y: i32,
        button: PointerButton,
    },
    Text {
        value: String,
    },
    Submit,
}

impl fmt::Debug for BrowserInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key { .. } => f.write_str("BrowserInput::Key([REDACTED])"),
            Self::Pointer { x, y, button } => f
                .debug_struct("BrowserInput::Pointer")
                .field("x", x)
                .field("y", y)
                .field("button", button)
                .finish(),
            Self::Text { .. } => f.write_str("BrowserInput::Text([REDACTED])"),
            Self::Submit => f.write_str("BrowserInput::Submit"),
        }
    }
}

/// Versionable, transport-neutral commands.  Implementations may map these
/// to IPC/WebSocket messages later; the commands contain no site semantics or
/// server filesystem authority.
#[derive(Clone, Eq, PartialEq)]
pub enum BrowserCommand {
    OpenSession { mode: BrowserAuthMode },
    AttachProfile { profile: ProfileAttachmentRef },
    DetachProfile,
    Navigate { request: BrowserNavigationRequest },
    Input { input: BrowserInput },
    QueryStatus,
    PollEvents { after_sequence: u64 },
    Cancel { operation_id: BrowserOperationId },
    Close,
}

impl fmt::Debug for BrowserCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenSession { mode } => f
                .debug_struct("BrowserCommand::OpenSession")
                .field("mode", mode)
                .finish(),
            Self::AttachProfile { .. } => f.write_str("BrowserCommand::AttachProfile([OPAQUE])"),
            Self::DetachProfile => f.write_str("BrowserCommand::DetachProfile"),
            Self::Navigate { request } => f
                .debug_struct("BrowserCommand::Navigate")
                .field("request", request)
                .finish(),
            Self::Input { input } => f
                .debug_struct("BrowserCommand::Input")
                .field("input", input)
                .finish(),
            Self::QueryStatus => f.write_str("BrowserCommand::QueryStatus"),
            Self::PollEvents { after_sequence } => f
                .debug_struct("BrowserCommand::PollEvents")
                .field("after_sequence", after_sequence)
                .finish(),
            Self::Cancel { operation_id } => f
                .debug_struct("BrowserCommand::Cancel")
                .field("operation_id", operation_id)
                .finish(),
            Self::Close => f.write_str("BrowserCommand::Close"),
        }
    }
}

impl BrowserInput {
    pub(crate) fn kind(&self) -> InputKind {
        match self {
            Self::Key { .. } => InputKind::Key,
            Self::Pointer { .. } => InputKind::Pointer,
            Self::Text { .. } => InputKind::Text,
            Self::Submit => InputKind::Submit,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Auxiliary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputKind {
    Key,
    Pointer,
    Text,
    Submit,
}

/// Generic, versioned facts emitted by a Browser Worker.  There are no
/// selectors, private API names, login-success rules, or site media concepts.
#[derive(Clone, Eq, PartialEq)]
pub struct BrowserEvent {
    pub version: u16,
    pub sequence: u64,
    pub kind: BrowserEventKind,
}

impl fmt::Debug for BrowserEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserEvent")
            .field("version", &self.version)
            .field("sequence", &self.sequence)
            .field("kind", &self.kind)
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum BrowserEventKind {
    WorkerOpened { session: BrowserSessionId },
    ProfileAttached,
    ProfileDetached,
    NavigationStarted { url: Url },
    NavigationChanged { url: Url, title: Option<String> },
    ResourceObserved { candidate: BrowserMediaCandidate },
    Loading,
    Ready,
    InputAccepted { kind: InputKind },
    InputResult { kind: InputKind, accepted: bool },
    NetworkDenied,
    Error { code: BrowserError },
    OperationCancelled { operation_id: BrowserOperationId },
    WorkerClosed,
    WorkerCrashed,
    WorkerTimedOut,
}

impl fmt::Debug for BrowserEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerOpened { session } => f
                .debug_struct("BrowserEventKind::WorkerOpened")
                .field("session", session)
                .finish(),
            Self::ProfileAttached => f.write_str("BrowserEventKind::ProfileAttached"),
            Self::ProfileDetached => f.write_str("BrowserEventKind::ProfileDetached"),
            Self::NavigationStarted { url } => {
                let safe_url = redacted_url(url);
                f.debug_struct("BrowserEventKind::NavigationStarted")
                    .field("url", &safe_url)
                    .finish()
            }
            Self::NavigationChanged { url, title } => {
                let safe_url = redacted_url(url);
                let safe_title = title.as_ref().map(|_| "[REDACTED]");
                f.debug_struct("BrowserEventKind::NavigationChanged")
                    .field("url", &safe_url)
                    .field("title", &safe_title)
                    .finish()
            }
            Self::ResourceObserved { candidate } => f
                .debug_struct("BrowserEventKind::ResourceObserved")
                .field("candidate", candidate)
                .finish(),
            Self::Loading => f.write_str("BrowserEventKind::Loading"),
            Self::Ready => f.write_str("BrowserEventKind::Ready"),
            Self::InputAccepted { kind } => f
                .debug_struct("BrowserEventKind::InputAccepted")
                .field("kind", kind)
                .finish(),
            Self::InputResult { kind, accepted } => f
                .debug_struct("BrowserEventKind::InputResult")
                .field("kind", kind)
                .field("accepted", accepted)
                .finish(),
            Self::NetworkDenied => f.write_str("BrowserEventKind::NetworkDenied"),
            Self::Error { code } => f
                .debug_struct("BrowserEventKind::Error")
                .field("code", code)
                .finish(),
            Self::OperationCancelled { operation_id } => f
                .debug_struct("BrowserEventKind::OperationCancelled")
                .field("operation_id", operation_id)
                .finish(),
            Self::WorkerClosed => f.write_str("BrowserEventKind::WorkerClosed"),
            Self::WorkerCrashed => f.write_str("BrowserEventKind::WorkerCrashed"),
            Self::WorkerTimedOut => f.write_str("BrowserEventKind::WorkerTimedOut"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelFeature {
    Clipboard,
    FileUpload,
    Audio,
}

/// Panel permissions are deny-by-default.  There is intentionally no public
/// constructor that grants these features.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PanelPermissions;

impl PanelPermissions {
    pub const fn allows(self, _feature: PanelFeature) -> bool {
        false
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PanelSessionId(String);

impl PanelSessionId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4().simple().to_string())
    }
}

impl fmt::Debug for PanelSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PanelSessionId").field(&"[opaque]").finish()
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PanelControlToken(String);

impl fmt::Debug for PanelControlToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PanelControlToken")
            .field(&"[redacted]")
            .finish()
    }
}

#[derive(Clone)]
pub struct NativePanelSession {
    id: PanelSessionId,
    worker_session: BrowserSessionId,
    token: PanelControlToken,
    expires_at: Instant,
    permissions: PanelPermissions,
}

impl fmt::Debug for NativePanelSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativePanelSession")
            .field("id", &self.id)
            .field("worker_session", &self.worker_session)
            .field("token", &self.token)
            .field("expires_at", &"[short-lived]")
            .field("permissions", &self.permissions)
            .finish()
    }
}

impl NativePanelSession {
    pub fn worker_session(&self) -> &BrowserSessionId {
        &self.worker_session
    }

    pub fn permissions(&self) -> PanelPermissions {
        self.permissions
    }

    pub(crate) fn id(&self) -> &PanelSessionId {
        &self.id
    }

    pub(crate) fn token(&self) -> &PanelControlToken {
        &self.token
    }

    pub(crate) fn expires_at(&self) -> Instant {
        self.expires_at
    }

    pub(crate) fn new_for_worker(worker_session: BrowserSessionId, ttl: Duration) -> Self {
        Self {
            id: PanelSessionId::new(),
            worker_session,
            token: PanelControlToken(Uuid::new_v4().simple().to_string()),
            expires_at: Instant::now() + ttl,
            permissions: PanelPermissions,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_wrong_token(&self) -> Self {
        Self {
            id: self.id.clone(),
            worker_session: self.worker_session.clone(),
            token: PanelControlToken(Uuid::new_v4().simple().to_string()),
            expires_at: self.expires_at,
            permissions: self.permissions,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PanelStatus {
    Connected,
    Disconnected,
}

#[derive(Clone, Debug)]
struct PanelRecord {
    worker_session: BrowserSessionId,
    token: PanelControlToken,
    expires_at: Instant,
    status: PanelStatus,
}

#[derive(Clone, Debug)]
struct FakeSession {
    status: BrowserStatus,
    events: VecDeque<BrowserEvent>,
    sequence: u64,
    profile: Option<ProfileAttachmentRef>,
    current_page_url: Option<Url>,
    current_page_title: String,
    observations: HashMap<BrowserOperationId, BrowserObservationPayload>,
    #[cfg(test)]
    next_observation: Option<BrowserObservationPayload>,
    candidate_material: Option<BrowserCandidateMaterial>,
}

#[derive(Debug, Default)]
struct FakeState {
    sessions: HashMap<BrowserSessionId, FakeSession>,
    profiles: HashMap<ProfileAttachmentRef, Instant>,
    panels: HashMap<PanelSessionId, PanelRecord>,
    cancelled: HashSet<BrowserOperationId>,
}

/// In-memory deterministic worker used by contract tests and future hosted
/// harnesses.  It does not launch a browser process.
#[derive(Clone, Debug, Default)]
pub struct FakeBrowserWorker {
    state: Arc<Mutex<FakeState>>,
}

impl FakeBrowserWorker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the next server-internal candidate capture for a fake
    /// session. This helper is crate-visible test infrastructure only.
    #[cfg(test)]
    pub(crate) fn set_candidate_material(
        &self,
        session: &BrowserSessionId,
        material: BrowserCandidateMaterial,
    ) -> Result<(), BrowserError> {
        let mut state = self.lock_state()?;
        let session = Self::session_mut(&mut state, session)?;
        Self::ensure_open(session)?;
        session.candidate_material = Some(material);
        Ok(())
    }

    /// Test-only issuance of an opaque profile reference.  Production refs
    /// are issued by Core/Vault infrastructure instead.
    #[cfg(test)]
    fn issue_profile_attachment(&self, ttl: Duration) -> ProfileAttachmentRef {
        let reference = ProfileAttachmentRef(Uuid::new_v4().simple().to_string());
        self.state
            .lock()
            .expect("fake browser state poisoned")
            .profiles
            .insert(reference.clone(), Instant::now() + ttl);
        reference
    }

    pub fn open_panel(
        &self,
        session: &BrowserSessionId,
        ttl: Duration,
    ) -> Result<NativePanelSession, BrowserError> {
        let mut state = self.lock_state()?;
        let worker_session = state
            .sessions
            .get(session)
            .filter(|session| session.status == BrowserStatus::Open)
            .map(|_| session.clone())
            .ok_or(BrowserError::InvalidSession)?;
        let id = PanelSessionId::new();
        let token = PanelControlToken(Uuid::new_v4().simple().to_string());
        let expires_at = Instant::now() + ttl;
        state.panels.insert(
            id.clone(),
            PanelRecord {
                worker_session: worker_session.clone(),
                token: token.clone(),
                expires_at,
                status: PanelStatus::Connected,
            },
        );
        Ok(NativePanelSession {
            id,
            worker_session,
            token,
            expires_at,
            permissions: PanelPermissions,
        })
    }

    pub fn panel_control(
        &self,
        panel: &NativePanelSession,
        input: BrowserInput,
    ) -> Result<(), BrowserError> {
        let mut state = self.lock_state()?;
        let record = state
            .panels
            .get_mut(&panel.id)
            .ok_or(BrowserError::SessionExpired)?;
        if record.expires_at <= Instant::now() || panel.expires_at <= Instant::now() {
            state.panels.remove(&panel.id);
            return Err(BrowserError::SessionExpired);
        }
        if record.worker_session != panel.worker_session || record.token != panel.token {
            return Err(BrowserError::SessionExpired);
        }
        if record.status == PanelStatus::Disconnected {
            return Err(BrowserError::PanelDisconnected);
        }
        drop(state);
        self.send_input(&panel.worker_session, input)
            .now_or_never()
            .unwrap_or(Err(BrowserError::WorkerUnavailable))
    }

    pub fn disconnect_panel(&self, panel: &NativePanelSession) -> Result<(), BrowserError> {
        let mut state = self.lock_state()?;
        let record = state
            .panels
            .get_mut(&panel.id)
            .ok_or(BrowserError::SessionExpired)?;
        if record.token != panel.token {
            return Err(BrowserError::SessionExpired);
        }
        record.status = PanelStatus::Disconnected;
        Ok(())
    }

    pub fn reconnect_panel(
        &self,
        panel: &NativePanelSession,
        ttl: Duration,
    ) -> Result<NativePanelSession, BrowserError> {
        let mut state = self.lock_state()?;
        let record = state
            .panels
            .get(&panel.id)
            .ok_or(BrowserError::SessionExpired)?;
        if record.token != panel.token {
            return Err(BrowserError::SessionExpired);
        }
        let worker_session = record.worker_session.clone();
        state
            .sessions
            .get(&worker_session)
            .filter(|session| session.status == BrowserStatus::Open)
            .ok_or(BrowserError::InvalidSession)?;
        let token = PanelControlToken(Uuid::new_v4().simple().to_string());
        let expires_at = Instant::now() + ttl;
        let record = state
            .panels
            .get_mut(&panel.id)
            .ok_or(BrowserError::SessionExpired)?;
        record.token = token.clone();
        record.expires_at = expires_at;
        record.status = PanelStatus::Connected;
        Ok(NativePanelSession {
            id: panel.id.clone(),
            worker_session,
            token,
            expires_at,
            permissions: PanelPermissions,
        })
    }

    pub fn crash(&self, session: &BrowserSessionId) -> Result<(), BrowserError> {
        self.terminate(session, BrowserStatus::Crashed)
    }

    pub fn timeout(&self, session: &BrowserSessionId) -> Result<(), BrowserError> {
        self.terminate(session, BrowserStatus::TimedOut)
    }

    /// Remove expired profile and panel capabilities.  This is intentionally
    /// explicit so a runtime can attach cleanup to its own lifecycle policy.
    pub fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let mut state = self.state.lock().expect("fake browser state poisoned");
        let before = state.profiles.len() + state.panels.len();
        state.profiles.retain(|_, expiry| *expiry > now);
        state.panels.retain(|_, panel| panel.expires_at > now);
        before - state.profiles.len() - state.panels.len()
    }

    pub fn profile_count(&self) -> usize {
        self.state
            .lock()
            .expect("fake browser state poisoned")
            .profiles
            .len()
    }

    pub fn panel_count(&self) -> usize {
        self.state
            .lock()
            .expect("fake browser state poisoned")
            .panels
            .len()
    }

    /// Test-only server-owned observation injection. The operation id is
    /// rebound to the next navigation by the fake worker; production workers
    /// populate the same ledger from real browser network events.
    #[cfg(test)]
    pub(crate) fn set_next_observation(
        &self,
        session: &BrowserSessionId,
        payload: BrowserObservationPayload,
    ) -> Result<(), BrowserError> {
        validate_browser_observation(&payload.observation)
            .map_err(|_| BrowserError::InvalidInput)?;
        validate_server_owned_observation(&payload.server_observation)
            .map_err(|_| BrowserError::InvalidInput)?;
        let mut state = self.lock_state()?;
        let session_state = Self::session_mut(&mut state, session)?;
        Self::ensure_open(session_state)?;
        session_state.next_observation = Some(payload);
        Ok(())
    }

    #[cfg(test)]
    fn publish_observation(
        &self,
        session: &BrowserSessionId,
        payload: BrowserObservationPayload,
    ) -> Result<(), BrowserError> {
        validate_browser_observation(&payload.observation)
            .map_err(|_| BrowserError::InvalidInput)?;
        validate_server_owned_observation(&payload.server_observation)
            .map_err(|_| BrowserError::InvalidInput)?;
        let mut state = self.lock_state()?;
        let session_state = Self::session_mut(&mut state, session)?;
        Self::ensure_open(session_state)?;
        session_state
            .observations
            .insert(payload.operation_id, payload);
        Ok(())
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, FakeState>, BrowserError> {
        self.state
            .lock()
            .map_err(|_| BrowserError::WorkerUnavailable)
    }

    fn push_event(session: &mut FakeSession, kind: BrowserEventKind) {
        session.sequence += 1;
        if session.events.len() >= MAX_BROWSER_EVENTS {
            session.events.pop_front();
        }
        session.events.push_back(BrowserEvent {
            version: BROWSER_EVENT_VERSION,
            sequence: session.sequence,
            kind,
        });
    }

    fn session_mut<'a>(
        state: &'a mut FakeState,
        session: &BrowserSessionId,
    ) -> Result<&'a mut FakeSession, BrowserError> {
        state
            .sessions
            .get_mut(session)
            .ok_or(BrowserError::InvalidSession)
    }

    fn ensure_open(session: &FakeSession) -> Result<(), BrowserError> {
        match session.status {
            BrowserStatus::Open => Ok(()),
            BrowserStatus::Closed => Err(BrowserError::SessionClosed),
            BrowserStatus::Crashed => Err(BrowserError::WorkerCrashed),
            BrowserStatus::TimedOut => Err(BrowserError::WorkerTimeout),
        }
    }

    fn terminate(
        &self,
        session: &BrowserSessionId,
        status: BrowserStatus,
    ) -> Result<(), BrowserError> {
        let mut state = self.lock_state()?;
        {
            let session_state = Self::session_mut(&mut state, session)?;
            Self::ensure_open(session_state)?;
            session_state.status = status;
            session_state.profile = None;
            session_state.observations.clear();
            Self::push_event(
                session_state,
                match status {
                    BrowserStatus::Crashed => BrowserEventKind::WorkerCrashed,
                    BrowserStatus::TimedOut => BrowserEventKind::WorkerTimedOut,
                    _ => BrowserEventKind::WorkerClosed,
                },
            );
        }
        for panel in state.panels.values_mut() {
            if &panel.worker_session == session {
                panel.status = PanelStatus::Disconnected;
            }
        }
        Ok(())
    }
}

/// Target-neutral worker lifecycle contract.  Implementations may be backed
/// by Chromium later, but the contract itself has no process/runtime policy.
pub trait BrowserWorker: Send + Sync {
    fn open_session(&self, mode: BrowserAuthMode) -> BrowserFuture<'_, BrowserSession>;
    fn attach_profile(
        &self,
        session: &BrowserSessionId,
        profile: ProfileAttachmentRef,
    ) -> BrowserFuture<'_, ()>;

    /// Attach a Vault-owned profile capability after materializing it into the
    /// worker's already-fresh disposable profile directory.  Implementations
    /// that do not need a filesystem-backed profile may use the legacy
    /// `attach_profile` path; real runtimes must consume the capability once.
    fn attach_profile_with_materializer(
        &self,
        session: &BrowserSessionId,
        profile: ProfileAttachmentRef,
        materializer: Arc<dyn ProfileMaterializer>,
    ) -> BrowserFuture<'_, ()> {
        let _ = materializer;
        self.attach_profile(session, profile)
    }
    /// Capture bounded server-owned material from a live auth session. The
    /// default is fail-closed for workers that do not implement auth capture.
    fn capture_candidate_material(
        &self,
        _session: &BrowserSessionId,
    ) -> BrowserFuture<'_, BrowserCandidateMaterial> {
        Box::pin(async { Err(BrowserError::CandidateCaptureUnavailable) })
    }
    fn detach_profile(&self, session: &BrowserSessionId) -> BrowserFuture<'_, ()>;
    fn navigate<'a>(
        &'a self,
        session: &'a BrowserSessionId,
        request: BrowserNavigationRequest,
        policy: &'a R008NavigationPolicy,
    ) -> BrowserFuture<'a, ()>;
    fn send_input(&self, session: &BrowserSessionId, input: BrowserInput) -> BrowserFuture<'_, ()>;
    fn status(&self, session: &BrowserSessionId) -> Result<BrowserStatus, BrowserError>;
    fn poll_events(
        &self,
        session: &BrowserSessionId,
        after_sequence: u64,
    ) -> Result<Vec<BrowserEvent>, BrowserError>;
    fn take_observation(
        &self,
        session: &BrowserSessionId,
        operation_id: BrowserOperationId,
    ) -> Result<Option<BrowserObservationPayload>, BrowserError>;
    fn cancel(
        &self,
        session: &BrowserSessionId,
        operation_id: BrowserOperationId,
    ) -> Result<(), BrowserError>;
    fn close(&self, session: &BrowserSessionId) -> Result<(), BrowserError>;
}

/// Server-side profile materialization seam.  The reference is opaque and the
/// destination is owned by a disposable worker runtime; neither is exposed to
/// plugins, Control, Display, or ordinary browser events.
pub trait ProfileMaterializer: Send + Sync {
    fn materialize(
        &self,
        attachment: &ProfileAttachmentRef,
        destination: &Path,
    ) -> Result<(), BrowserError>;
}

impl BrowserWorker for FakeBrowserWorker {
    fn open_session(&self, mode: BrowserAuthMode) -> BrowserFuture<'_, BrowserSession> {
        Box::pin(async move {
            let mut state = self.lock_state()?;
            let session = BrowserSession {
                id: BrowserSessionId::new(),
                mode,
            };
            let mut fake = FakeSession {
                status: BrowserStatus::Open,
                events: VecDeque::new(),
                sequence: 0,
                profile: None,
                current_page_url: None,
                current_page_title: String::new(),
                observations: HashMap::new(),
                #[cfg(test)]
                next_observation: None,
                candidate_material: None,
            };
            Self::push_event(
                &mut fake,
                BrowserEventKind::WorkerOpened {
                    session: session.id.clone(),
                },
            );
            state.sessions.insert(session.id.clone(), fake);
            Ok(session)
        })
    }

    fn attach_profile(
        &self,
        session: &BrowserSessionId,
        profile: ProfileAttachmentRef,
    ) -> BrowserFuture<'_, ()> {
        let session = session.clone();
        Box::pin(async move {
            let mut state = self.lock_state()?;
            let expiry = state
                .profiles
                .get(&profile)
                .copied()
                .ok_or(BrowserError::ProfileAttachFailed)?;
            if expiry <= Instant::now() {
                state.profiles.remove(&profile);
                return Err(BrowserError::SessionExpired);
            }
            let session_state = Self::session_mut(&mut state, &session)?;
            Self::ensure_open(session_state)?;
            session_state.profile = Some(profile);
            Self::push_event(session_state, BrowserEventKind::ProfileAttached);
            Ok(())
        })
    }

    fn attach_profile_with_materializer(
        &self,
        session: &BrowserSessionId,
        profile: ProfileAttachmentRef,
        materializer: Arc<dyn ProfileMaterializer>,
    ) -> BrowserFuture<'_, ()> {
        let session = session.clone();
        Box::pin(async move {
            let destination = std::env::temp_dir().join(format!(
                "web-media-gateway-fake-profile-{}",
                Uuid::new_v4().simple()
            ));
            std::fs::create_dir(&destination).map_err(|_| BrowserError::ProfileAttachFailed)?;
            let result = materializer.materialize(&profile, &destination);
            let _ = std::fs::remove_dir_all(&destination);
            result?;

            let mut state = self.lock_state()?;
            let session_state = Self::session_mut(&mut state, &session)?;
            Self::ensure_open(session_state)?;
            session_state.profile = Some(profile);
            Self::push_event(session_state, BrowserEventKind::ProfileAttached);
            Ok(())
        })
    }

    fn capture_candidate_material(
        &self,
        session: &BrowserSessionId,
    ) -> BrowserFuture<'_, BrowserCandidateMaterial> {
        let session = session.clone();
        Box::pin(async move {
            let mut state = self.lock_state()?;
            let session = Self::session_mut(&mut state, &session)?;
            Self::ensure_open(session)?;
            session
                .candidate_material
                .take()
                .ok_or(BrowserError::CandidateCaptureUnavailable)
        })
    }

    fn detach_profile(&self, session: &BrowserSessionId) -> BrowserFuture<'_, ()> {
        let session = session.clone();
        Box::pin(async move {
            let mut state = self.lock_state()?;
            let session_state = Self::session_mut(&mut state, &session)?;
            Self::ensure_open(session_state)?;
            session_state.profile = None;
            Self::push_event(session_state, BrowserEventKind::ProfileDetached);
            Ok(())
        })
    }

    fn navigate<'a>(
        &'a self,
        session: &'a BrowserSessionId,
        request: BrowserNavigationRequest,
        policy: &'a R008NavigationPolicy,
    ) -> BrowserFuture<'a, ()> {
        Box::pin(async move {
            if policy.authorize(&request).await.is_err() {
                let mut state = self.lock_state()?;
                let session_state = Self::session_mut(&mut state, session)?;
                Self::push_event(session_state, BrowserEventKind::NetworkDenied);
                return Err(BrowserError::NavigationDenied);
            }
            let mut state = self.lock_state()?;
            if state.cancelled.remove(&request.operation_id()) {
                let session_state = Self::session_mut(&mut state, session)?;
                Self::push_event(
                    session_state,
                    BrowserEventKind::OperationCancelled {
                        operation_id: request.operation_id(),
                    },
                );
                return Err(BrowserError::OperationCancelled);
            }
            let session_state = Self::session_mut(&mut state, session)?;
            Self::ensure_open(session_state)?;
            Self::push_event(
                session_state,
                BrowserEventKind::NavigationStarted {
                    url: redacted_event_url(request.url()),
                },
            );
            Self::push_event(session_state, BrowserEventKind::Loading);
            Self::push_event(
                session_state,
                BrowserEventKind::NavigationChanged {
                    url: redacted_event_url(request.url()),
                    title: None,
                },
            );
            session_state.current_page_url = Some(redacted_event_url(request.url()));
            session_state.current_page_title.clear();
            Self::push_event(session_state, BrowserEventKind::Ready);
            #[cfg(test)]
            if let Some(mut payload) = session_state.next_observation.take() {
                payload.operation_id = request.operation_id();
                session_state
                    .observations
                    .insert(request.operation_id(), payload);
            }
            Ok(())
        })
    }

    fn send_input(&self, session: &BrowserSessionId, input: BrowserInput) -> BrowserFuture<'_, ()> {
        let session = session.clone();
        Box::pin(async move {
            let mut state = self.lock_state()?;
            let session_state = Self::session_mut(&mut state, &session)?;
            Self::ensure_open(session_state)?;
            let kind = input.kind();
            // Input payloads are intentionally not put into events or state;
            // this keeps passwords/codes/QR content out of normal artifacts.
            drop(input);
            Self::push_event(session_state, BrowserEventKind::InputAccepted { kind });
            Self::push_event(
                session_state,
                BrowserEventKind::InputResult {
                    kind,
                    accepted: true,
                },
            );
            Ok(())
        })
    }

    fn status(&self, session: &BrowserSessionId) -> Result<BrowserStatus, BrowserError> {
        let state = self.lock_state()?;
        state
            .sessions
            .get(session)
            .map(|session| session.status)
            .ok_or(BrowserError::InvalidSession)
    }

    fn poll_events(
        &self,
        session: &BrowserSessionId,
        after_sequence: u64,
    ) -> Result<Vec<BrowserEvent>, BrowserError> {
        let state = self.lock_state()?;
        let session = state
            .sessions
            .get(session)
            .ok_or(BrowserError::InvalidSession)?;
        Ok(session
            .events
            .iter()
            .filter(|event| event.sequence > after_sequence)
            .cloned()
            .collect())
    }

    fn take_observation(
        &self,
        session: &BrowserSessionId,
        operation_id: BrowserOperationId,
    ) -> Result<Option<BrowserObservationPayload>, BrowserError> {
        let mut state = self.lock_state()?;
        let session_state = Self::session_mut(&mut state, session)?;
        Self::ensure_open(session_state)?;
        Ok(session_state.observations.remove(&operation_id))
    }

    fn cancel(
        &self,
        session: &BrowserSessionId,
        operation_id: BrowserOperationId,
    ) -> Result<(), BrowserError> {
        let mut state = self.lock_state()?;
        {
            let session_state = Self::session_mut(&mut state, session)?;
            Self::ensure_open(session_state)?;
        }
        state.cancelled.insert(operation_id);
        Ok(())
    }

    fn close(&self, session: &BrowserSessionId) -> Result<(), BrowserError> {
        let mut state = self.lock_state()?;
        {
            let session_state = Self::session_mut(&mut state, session)?;
            Self::ensure_open(session_state)?;
            session_state.status = BrowserStatus::Closed;
            session_state.profile = None;
            session_state.observations.clear();
            Self::push_event(session_state, BrowserEventKind::WorkerClosed);
        }
        for panel in state.panels.values_mut() {
            if &panel.worker_session == session {
                panel.status = PanelStatus::Disconnected;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use site_adapter_api::{
        BrowserExpiryHint, BrowserMediaKind, BrowserRangeSupport, BrowserStatusClass,
        ServerOwnedMedia, StreamProtocol,
    };
    use std::collections::BTreeMap;

    fn policy() -> R008NavigationPolicy {
        R008NavigationPolicy::public_web(EgressPolicy::default())
    }

    async fn session(worker: &FakeBrowserWorker) -> BrowserSession {
        worker.open_session(BrowserAuthMode::Passive).await.unwrap()
    }

    #[tokio::test]
    async fn fake_worker_has_deterministic_generic_lifecycle_and_ordering() {
        let worker = FakeBrowserWorker::new();
        let session = session(&worker).await;
        let profile = worker.issue_profile_attachment(Duration::from_secs(30));
        worker.attach_profile(session.id(), profile).await.unwrap();
        worker
            .navigate(
                session.id(),
                BrowserNavigationRequest::new(Url::parse("https://1.1.1.1/video").unwrap()),
                &policy(),
            )
            .await
            .unwrap();
        worker
            .send_input(
                session.id(),
                BrowserInput::Text {
                    value: "password-sentinel-7f3e".into(),
                },
            )
            .await
            .unwrap();
        worker.close(session.id()).unwrap();

        let events = worker.poll_events(session.id(), 0).unwrap();
        assert!(!format!("{events:?}").contains("password-sentinel-7f3e"));
        assert_eq!(events.first().unwrap().version, BROWSER_EVENT_VERSION);
        assert!(matches!(
            events[0].kind,
            BrowserEventKind::WorkerOpened { .. }
        ));
        assert!(
            events
                .windows(2)
                .all(|pair| pair[0].sequence < pair[1].sequence)
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event.kind, BrowserEventKind::Ready))
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event.kind, BrowserEventKind::WorkerClosed))
        );
    }

    #[tokio::test]
    async fn observation_handoff_is_bounded_bound_and_one_shot() {
        let worker = FakeBrowserWorker::new();
        let session = session(&worker).await;
        let operation = BrowserNavigationRequest::new(
            Url::parse("https://example.test/watch?p=2&token=event-secret").unwrap(),
        )
        .operation_id();
        let observation_id = "observation-fixture".to_owned();
        let candidate = BrowserMediaCandidate {
            id: "resource-1".into(),
            kind: BrowserMediaKind::Muxed,
            group_id: None,
            codec: None,
            container: Some("mp4".into()),
            mime_type: Some("video/mp4".into()),
            width: None,
            height: None,
            bitrate: None,
            language: None,
            protocol: StreamProtocol::HttpFile,
            status: BrowserStatusClass::Success,
            range: BrowserRangeSupport::Supported,
            egress_allowed: true,
            access_ref: "opaque-media-ref".into(),
            expiry: BrowserExpiryHint::NoneObserved,
            expires_at: None,
        };
        worker
            .publish_observation(
                session.id(),
                BrowserObservationPayload {
                    operation_id: operation,
                    observation: BrowserObservation {
                        schema_version: site_adapter_api::BROWSER_OBSERVATION_VERSION,
                        observation_id: observation_id.clone(),
                        page_url: "https://example.test/watch?p=2".into(),
                        page_title: "fixture page".into(),
                        event_count: 3,
                        resource_count: 1,
                        candidates: vec![candidate],
                    },
                    server_observation: ServerOwnedObservation {
                        schema_version: site_adapter_api::BROWSER_OBSERVATION_VERSION,
                        observation_id,
                        media: vec![ServerOwnedMedia {
                            observation_id: "observation-fixture".into(),
                            candidate_id: "resource-1".into(),
                            access_ref: "opaque-media-ref".into(),
                            protocol: StreamProtocol::HttpFile,
                            url: Url::parse(
                                "https://cdn.example.test/media.mp4?signature=server-secret",
                            )
                            .unwrap(),
                            public_headers: BTreeMap::new(),
                        }],
                    },
                },
            )
            .unwrap();
        let locator = site_adapter_api::SourceLocator {
            site_id: "fixture-site".into(),
            plugin_id: "fixture-plugin".into(),
            locator_version: 1,
            opaque_payload: "opaque-locator".into(),
        };
        let handoff = BrowserObservationHandoff::take_from_worker(
            &worker,
            session.id(),
            operation,
            locator.clone(),
            BROWSER_HANDOFF_TTL,
        )
        .unwrap()
        .unwrap();
        assert_eq!(handoff.session_id(), session.id());
        assert_eq!(handoff.operation_id(), operation);
        assert_eq!(handoff.locator(), &locator);
        assert!(!handoff.is_expired());
        let diagnostics = format!("{handoff:?}");
        assert!(!diagnostics.contains("server-secret"));
        assert!(!diagnostics.contains("signature="));
        assert!(
            BrowserObservationHandoff::take_from_worker(
                &worker,
                session.id(),
                operation,
                locator,
                BROWSER_HANDOFF_TTL,
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn navigation_event_redaction_preserves_safe_page_part_without_signed_data() {
        let url =
            Url::parse("https://example.test/video?p=2&token=secret&signature=secret#private")
                .unwrap();
        let redacted = redacted_event_url(&url);
        assert_eq!(redacted.as_str(), "https://example.test/video?p=2");
    }

    #[tokio::test]
    async fn profile_refs_are_opaque_and_expiry_is_rejected_and_cleaned() {
        let worker = FakeBrowserWorker::new();
        let session = session(&worker).await;
        let expired = worker.issue_profile_attachment(Duration::ZERO);
        assert_eq!(worker.profile_count(), 1);
        assert_eq!(
            worker.attach_profile(session.id(), expired).await,
            Err(BrowserError::SessionExpired)
        );
        assert_eq!(worker.profile_count(), 0);
        let missing = ProfileAttachmentRef("not-a-vault-path".into());
        assert_eq!(
            worker.attach_profile(session.id(), missing).await,
            Err(BrowserError::ProfileAttachFailed)
        );
    }

    #[tokio::test]
    async fn navigation_uses_r008_policy_and_denies_local_file_and_private_targets() {
        let worker = FakeBrowserWorker::new();
        let session = session(&worker).await;
        let r008 = policy();
        for url in [
            "file:///etc/passwd",
            "http://127.0.0.1:8080/",
            "http://169.254.169.254/latest",
        ] {
            let result = worker
                .navigate(
                    session.id(),
                    BrowserNavigationRequest::new(Url::parse(url).unwrap()),
                    &r008,
                )
                .await;
            assert_eq!(result, Err(BrowserError::NavigationDenied), "{url}");
        }
        let events = worker.poll_events(session.id(), 0).unwrap();
        assert!(
            !events
                .iter()
                .any(|event| matches!(event.kind, BrowserEventKind::NavigationStarted { .. }))
        );
    }

    #[tokio::test]
    async fn configured_local_service_is_explicit_and_not_user_selectable() {
        let worker = FakeBrowserWorker::new();
        let session = session(&worker).await;
        let mut egress = EgressPolicy::default();
        egress
            .configure_local_service(
                "configured-test-service",
                &Url::parse("http://127.0.0.1:8096/").unwrap(),
            )
            .unwrap();
        let public = R008NavigationPolicy::public_web(egress.clone());
        let trusted_local =
            R008NavigationPolicy::configured_local_service(egress, "configured-test-service");
        assert!(
            worker
                .navigate(
                    session.id(),
                    BrowserNavigationRequest::new(
                        Url::parse("http://127.0.0.1:8096/health").unwrap(),
                    ),
                    &trusted_local,
                )
                .await
                .is_ok()
        );
        assert_eq!(
            worker
                .navigate(
                    session.id(),
                    BrowserNavigationRequest::new(
                        Url::parse("http://127.0.0.1:8096/health").unwrap(),
                    ),
                    &public,
                )
                .await,
            Err(BrowserError::NavigationDenied)
        );
    }

    #[tokio::test]
    async fn cancel_crash_timeout_and_close_have_explicit_outcomes_and_cleanup() {
        let worker = FakeBrowserWorker::new();
        let first = session(&worker).await;
        let request = BrowserNavigationRequest::new(Url::parse("https://1.1.1.1/cancel").unwrap());
        worker.cancel(first.id(), request.operation_id()).unwrap();
        assert_eq!(
            worker.navigate(first.id(), request, &policy()).await,
            Err(BrowserError::OperationCancelled)
        );
        worker.crash(first.id()).unwrap();
        assert_eq!(worker.status(first.id()), Ok(BrowserStatus::Crashed));
        assert_eq!(
            worker.send_input(first.id(), BrowserInput::Submit).await,
            Err(BrowserError::WorkerCrashed)
        );

        let second = session(&worker).await;
        worker.timeout(second.id()).unwrap();
        assert_eq!(worker.status(second.id()), Ok(BrowserStatus::TimedOut));
        assert_eq!(worker.close(second.id()), Err(BrowserError::WorkerTimeout));
    }

    #[tokio::test]
    async fn panel_is_bound_short_lived_reconnectable_and_deny_by_default() {
        let worker = FakeBrowserWorker::new();
        let session = session(&worker).await;
        let panel = worker
            .open_panel(session.id(), Duration::from_secs(30))
            .unwrap();
        assert!(!panel.permissions().allows(PanelFeature::Clipboard));
        assert!(!panel.permissions().allows(PanelFeature::FileUpload));
        assert!(!panel.permissions().allows(PanelFeature::Audio));
        worker
            .panel_control(
                &panel,
                BrowserInput::Pointer {
                    x: 10,
                    y: 20,
                    button: PointerButton::Primary,
                },
            )
            .unwrap();
        worker.disconnect_panel(&panel).unwrap();
        assert_eq!(
            worker.panel_control(&panel, BrowserInput::Submit),
            Err(BrowserError::PanelDisconnected)
        );
        let reconnected = worker
            .reconnect_panel(&panel, Duration::from_secs(30))
            .unwrap();
        assert_eq!(
            worker.panel_control(&panel, BrowserInput::Submit),
            Err(BrowserError::SessionExpired)
        );
        worker
            .panel_control(&reconnected, BrowserInput::Submit)
            .unwrap();
    }

    #[tokio::test]
    async fn panel_expiry_and_worker_failure_disconnect_and_cleanup() {
        let worker = FakeBrowserWorker::new();
        let session = session(&worker).await;
        let expired = worker.open_panel(session.id(), Duration::ZERO).unwrap();
        assert_eq!(
            worker.panel_control(&expired, BrowserInput::Submit),
            Err(BrowserError::SessionExpired)
        );
        assert_eq!(worker.panel_count(), 0);

        let panel = worker
            .open_panel(session.id(), Duration::from_secs(30))
            .unwrap();
        worker.timeout(session.id()).unwrap();
        assert_eq!(
            worker.panel_control(&panel, BrowserInput::Submit),
            Err(BrowserError::PanelDisconnected)
        );
    }

    #[tokio::test]
    async fn two_sessions_are_isolated() {
        let worker = FakeBrowserWorker::new();
        let one = session(&worker).await;
        let two = session(&worker).await;
        let profile = worker.issue_profile_attachment(Duration::from_secs(30));
        worker.attach_profile(one.id(), profile).await.unwrap();
        assert_eq!(worker.detach_profile(two.id()).await, Ok(()));
        let one_events = worker.poll_events(one.id(), 0).unwrap();
        let two_events = worker.poll_events(two.id(), 0).unwrap();
        assert!(
            one_events
                .iter()
                .any(|event| matches!(event.kind, BrowserEventKind::ProfileAttached))
        );
        assert!(
            !two_events
                .iter()
                .any(|event| matches!(event.kind, BrowserEventKind::ProfileAttached))
        );
    }

    #[test]
    fn debug_does_not_expose_profile_or_panel_token() {
        let profile = ProfileAttachmentRef("vault/path/cookie.sqlite".into());
        assert!(!format!("{profile:?}").contains("cookie.sqlite"));
        let token = PanelControlToken("fixture-panel-token".into());
        assert!(!format!("{token:?}").contains("fixture-panel-token"));
    }

    #[test]
    fn debug_does_not_expose_sensitive_input_or_command_payloads() {
        let sentinel = "verification-code-sentinel-93a1";
        let input = BrowserInput::Text {
            value: sentinel.into(),
        };
        let command = BrowserCommand::Input {
            input: input.clone(),
        };
        assert!(!format!("{input:?}").contains(sentinel));
        assert!(!format!("{command:?}").contains(sentinel));

        let profile = ProfileAttachmentRef("vault/cookie-db-sentinel".into());
        let attach = BrowserCommand::AttachProfile { profile };
        assert!(!format!("{attach:?}").contains("cookie-db-sentinel"));

        let request = BrowserNavigationRequest::new(
            Url::parse("https://1.1.1.1/video?token=url-secret-sentinel").unwrap(),
        );
        let navigate = BrowserCommand::Navigate { request };
        assert!(!format!("{navigate:?}").contains("url-secret-sentinel"));
    }

    #[test]
    fn debug_does_not_expose_sensitive_event_navigation_urls() {
        let sentinel = "event-secret-sentinel-7c42";
        let started = BrowserEvent {
            version: BROWSER_EVENT_VERSION,
            sequence: 1,
            kind: BrowserEventKind::NavigationStarted {
                url: Url::parse(&format!(
                    "https://example.test/watch?token={sentinel}#fragment"
                ))
                .unwrap(),
            },
        };
        let changed = BrowserEvent {
            version: BROWSER_EVENT_VERSION,
            sequence: 2,
            kind: BrowserEventKind::NavigationChanged {
                url: Url::parse(&format!(
                    "https://example.test/watch?token={sentinel}#fragment"
                ))
                .unwrap(),
                title: Some("event-title-sentinel".into()),
            },
        };
        let diagnostics = format!("{started:?} {changed:?}");
        assert!(!diagnostics.contains(sentinel));
        assert!(!diagnostics.contains("event-title-sentinel"));
        assert!(diagnostics.contains("https://example.test/watch"));
        assert!(diagnostics.contains("[REDACTED]"));
    }
}
