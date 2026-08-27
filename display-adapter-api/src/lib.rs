use futures_util::future::BoxFuture;
use std::fmt;
use url::Url;

/// Identity supplied by Playback Coordinator to bind every adapter operation.
/// An adapter may report candidate state for this identity, but it cannot commit
/// active_display or display_generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisplayContext {
    pub session_id: String,
    pub item_id: String,
    pub item_revision: u64,
    pub display_id: String,
    pub display_generation: u64,
}

impl DisplayContext {
    pub fn new(
        session_id: impl Into<String>,
        item_id: impl Into<String>,
        item_revision: u64,
        display_id: impl Into<String>,
        display_generation: u64,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            item_id: item_id.into(),
            item_revision,
            display_id: display_id.into(),
            display_generation,
        }
    }
}

/// An opaque, Gateway-issued media capability. It intentionally carries no
/// upstream headers, cookies, or authorization material.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayMediaCapability {
    url: Url,
    pub context: DisplayContext,
}

impl GatewayMediaCapability {
    pub fn from_gateway(url: Url, context: DisplayContext) -> Result<Self, CapabilityError> {
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || url.username() != ""
            || url.password().is_some()
            || url.fragment().is_some()
            || url.query_pairs().any(|(name, _)| {
                matches!(
                    name.to_ascii_lowercase().as_str(),
                    "cookie" | "authorization" | "proxy-authorization"
                )
            })
        {
            return Err(CapabilityError::InvalidUrl);
        }
        Ok(Self { url, context })
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityError {
    InvalidUrl,
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CapabilityError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisplayInstance {
    pub id: String,
    pub adapter_type: String,
    pub label: String,
    pub online: bool,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeRequest {
    pub context: DisplayContext,
    pub media: GatewayMediaCapability,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrepareRequest {
    pub context: DisplayContext,
    pub media: GatewayMediaCapability,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartRequest {
    pub context: DisplayContext,
    pub media: GatewayMediaCapability,
    pub position_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PositionSample {
    pub requested_ms: Option<u64>,
    pub reported_ms: u64,
    pub error_ms: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlaybackObservation {
    Playing,
    Paused,
    Stopped,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisplayStatus {
    pub context: DisplayContext,
    pub observation: PlaybackObservation,
    pub position: PositionSample,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrepareResult {
    pub context: DisplayContext,
    pub target: DisplayInstance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartResult {
    pub status: DisplayStatus,
    pub command_accepted: bool,
    pub playback_confirmed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisplayCommand {
    Pause,
    Resume,
    Seek { position_ms: u64 },
    Stop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisplayAdapterError {
    InvalidConfiguration,
    ServerUnavailable,
    AuthenticationFailed,
    TargetMissing,
    TargetOffline,
    TargetAmbiguous,
    MediaIncompatible,
    CommandRejected,
    PlaybackNotConfirmed { timeout_ms: u64 },
    Timeout,
    Cancelled,
    StaleContext,
    Protocol(String),
}

impl fmt::Display for DisplayAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DisplayAdapterError {}

pub trait DisplayAdapter: Send + Sync {
    fn adapter_type(&self) -> &'static str;
    fn list_or_register_displays<'a>(
        &'a self,
    ) -> BoxFuture<'a, Result<Vec<DisplayInstance>, DisplayAdapterError>>;
    fn probe<'a>(
        &'a self,
        request: ProbeRequest,
    ) -> BoxFuture<'a, Result<DisplayInstance, DisplayAdapterError>>;
    fn prepare<'a>(
        &'a self,
        request: PrepareRequest,
    ) -> BoxFuture<'a, Result<PrepareResult, DisplayAdapterError>>;
    fn start<'a>(
        &'a self,
        request: StartRequest,
    ) -> BoxFuture<'a, Result<StartResult, DisplayAdapterError>>;
    fn command<'a>(
        &'a self,
        context: DisplayContext,
        command: DisplayCommand,
    ) -> BoxFuture<'a, Result<DisplayStatus, DisplayAdapterError>>;
    fn status<'a>(
        &'a self,
        context: DisplayContext,
    ) -> BoxFuture<'a, Result<DisplayStatus, DisplayAdapterError>>;
}
