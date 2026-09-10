//! Gateway-owned delivery for validated separated audio/video media.
//!
//! This module deliberately stops at the generic `MediaShapeV1` boundary. A
//! Site Plugin supplies only bounded track metadata and server-owned
//! capabilities. The broker turns those capabilities into controlled local
//! inputs; FFmpeg is given only those local inputs and structured arguments.
//! The browser receives a short-lived, session-bound Gateway path for the
//! resulting fMP4 file.

use crate::UpstreamResource;
use crate::security::{EgressPolicy, StructuredCommand};
use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use site_adapter_api::{MediaShapeV1, MediaTrackKind, StreamProtocol};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use url::Url;
use uuid::Uuid;

pub const DELIVERY_CONTRACT_VERSION: u32 = 1;
pub const DEFAULT_DELIVERY_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_DELIVERY_TTL: Duration = Duration::from_secs(60);
pub const MAX_DELIVERY_TIMEOUT: Duration = Duration::from_secs(5 * 60);
pub const MAX_DELIVERY_TTL: Duration = Duration::from_secs(10 * 60);
pub const MAX_DELIVERY_OUTPUT_BYTES: u64 = 128 * 1024 * 1024;
const MAX_DELIVERY_ID_BYTES: usize = 128;
const MAX_DELIVERY_GROUP_BYTES: usize = 128;

/// The full CAS identity of one delivery attempt. No field is derived from a
/// short-lived URL or from browser-provided state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryBinding {
    pub session_id: String,
    pub item_id: String,
    pub item_revision: u64,
    pub media_generation: u64,
    pub display_generation: u64,
    pub group_id: String,
}

impl DeliveryBinding {
    pub fn new(
        session_id: impl Into<String>,
        item_id: impl Into<String>,
        item_revision: u64,
        media_generation: u64,
        display_generation: u64,
        group_id: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            item_id: item_id.into(),
            item_revision,
            media_generation,
            display_generation,
            group_id: group_id.into(),
        }
    }
}

/// A capability issued by server-side resolution. Its resource contains the
/// upstream URL and possibly secret headers, so the fields stay private and
/// its Debug implementation redacts the entire resource.
#[derive(Clone)]
pub struct DeliveryInputCapability {
    track_id: String,
    access_ref: String,
    resource: UpstreamResource,
}

impl fmt::Debug for DeliveryInputCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeliveryInputCapability")
            .field("track_id", &self.track_id)
            .field("access_ref", &"<redacted>")
            .field("resource", &"<server-only>")
            .finish()
    }
}

impl DeliveryInputCapability {
    /// Construct a capability only at a server-owned handoff boundary. The
    /// caller must have obtained `resource` through the normal resolver and
    /// EgressPolicy path; `start` repeats policy validation before use.
    pub fn new_server_owned(
        track_id: impl Into<String>,
        access_ref: impl Into<String>,
        resource: UpstreamResource,
    ) -> Result<Self, DeliveryError> {
        let capability = Self {
            track_id: track_id.into(),
            access_ref: access_ref.into(),
            resource,
        };
        if !valid_identifier(&capability.track_id)
            || !valid_opaque_ref(&capability.access_ref)
            || !valid_upstream_url(&capability.resource.url)
        {
            return Err(DeliveryError::InputCapabilityRejected);
        }
        Ok(capability)
    }

    pub fn track_id(&self) -> &str {
        &self.track_id
    }
}

/// A broker may materialize an input into a controlled local file or another
/// broker-owned file descriptor. The path is never accepted from a request or
/// put into a public DTO; it is consumed only by the structured FFmpeg argv.
pub struct BrokerInput {
    path: PathBuf,
}

impl fmt::Debug for BrokerInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrokerInput")
            .field("path", &"<broker-owned>")
            .finish()
    }
}

impl BrokerInput {
    pub fn from_server_owned_path(path: PathBuf) -> Result<Self, DeliveryError> {
        if path.as_os_str().is_empty() {
            return Err(DeliveryError::BrokerRejected);
        }
        Ok(Self { path })
    }
}

/// This is the only input seam used by the delivery process. A production
/// implementation can use the Gateway HTTP broker or a controlled stdin/fd
/// broker; neither URL nor headers are passed to FFmpeg by this trait.
pub trait DeliveryInputBroker: Send + Sync {
    fn acquire(
        &self,
        capability: &DeliveryInputCapability,
        workspace: &Path,
    ) -> Result<BrokerInput, DeliveryError>;
}

/// The authority checks the PlaybackSession snapshot before and after the
/// external process. A stale result is discarded before it can be projected.
pub trait DeliveryAuthority: Send + Sync {
    fn matches(&self, binding: &DeliveryBinding) -> bool;
}

#[derive(Clone, Default)]
pub struct DeliveryCancellation(Arc<AtomicBool>);

impl DeliveryCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Clone)]
pub struct DeliveryRequest {
    pub binding: DeliveryBinding,
    pub shape: MediaShapeV1,
    pub inputs: Vec<DeliveryInputCapability>,
    pub authority: Arc<dyn DeliveryAuthority>,
    pub timeout: Duration,
    pub output_ttl: Duration,
    pub max_output_bytes: u64,
}

impl fmt::Debug for DeliveryRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeliveryRequest")
            .field("binding", &self.binding)
            .field("shape", &self.shape)
            .field("inputs", &self.inputs)
            .field("timeout", &self.timeout)
            .field("output_ttl", &self.output_ttl)
            .field("max_output_bytes", &self.max_output_bytes)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryResult {
    pub contract_version: u32,
    pub binding: DeliveryBinding,
    /// Same-origin Gateway path only. It contains no upstream URL, header or
    /// server access reference.
    pub gateway_path: String,
    pub content_type: &'static str,
    pub output_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryError {
    InvalidRequest,
    UnsupportedShape,
    IncompleteTrackGroup,
    StaleGeneration,
    ExpiredInput,
    InputCapabilityRejected,
    EgressRejected,
    SecretMaterial,
    BrokerRejected,
    Cancelled,
    TimedOut,
    OutputLimitExceeded,
    ProcessFailed,
    CleanupFailed,
    OutputUnavailable,
    CapabilityExpired,
    CapabilityBindingMismatch,
    MethodNotAllowed,
    InvalidRange,
}

impl fmt::Display for DeliveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::InvalidRequest => "DELIVERY_REQUEST_INVALID",
            Self::UnsupportedShape => "DELIVERY_SHAPE_UNSUPPORTED",
            Self::IncompleteTrackGroup => "DELIVERY_TRACK_GROUP_INCOMPLETE",
            Self::StaleGeneration => "DELIVERY_STALE_GENERATION",
            Self::ExpiredInput => "DELIVERY_INPUT_EXPIRED",
            Self::InputCapabilityRejected => "DELIVERY_INPUT_CAPABILITY_REJECTED",
            Self::EgressRejected => "DELIVERY_EGRESS_REJECTED",
            Self::SecretMaterial => "DELIVERY_SECRET_REJECTED",
            Self::BrokerRejected => "DELIVERY_BROKER_REJECTED",
            Self::Cancelled => "DELIVERY_CANCELLED",
            Self::TimedOut => "DELIVERY_TIMEOUT",
            Self::OutputLimitExceeded => "DELIVERY_OUTPUT_LIMIT",
            Self::ProcessFailed => "DELIVERY_PROCESS_FAILED",
            Self::CleanupFailed => "DELIVERY_CLEANUP_FAILED",
            Self::OutputUnavailable => "DELIVERY_OUTPUT_UNAVAILABLE",
            Self::CapabilityExpired => "DELIVERY_CAPABILITY_EXPIRED",
            Self::CapabilityBindingMismatch => "DELIVERY_CAPABILITY_BINDING_MISMATCH",
            Self::MethodNotAllowed => "DELIVERY_METHOD_NOT_ALLOWED",
            Self::InvalidRange => "DELIVERY_RANGE_INVALID",
        };
        formatter.write_str(code)
    }
}

impl std::error::Error for DeliveryError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryFailureClass {
    RefreshRequired,
    Stale,
    Cancelled,
    TimedOut,
    Rejected,
}

impl DeliveryError {
    pub fn class(&self) -> DeliveryFailureClass {
        match self {
            Self::ExpiredInput | Self::CapabilityExpired => DeliveryFailureClass::RefreshRequired,
            Self::StaleGeneration | Self::CapabilityBindingMismatch => DeliveryFailureClass::Stale,
            Self::Cancelled => DeliveryFailureClass::Cancelled,
            Self::TimedOut => DeliveryFailureClass::TimedOut,
            Self::InvalidRequest
            | Self::UnsupportedShape
            | Self::IncompleteTrackGroup
            | Self::InputCapabilityRejected
            | Self::EgressRejected
            | Self::SecretMaterial
            | Self::BrokerRejected
            | Self::OutputLimitExceeded
            | Self::ProcessFailed
            | Self::CleanupFailed
            | Self::OutputUnavailable
            | Self::MethodNotAllowed
            | Self::InvalidRange => DeliveryFailureClass::Rejected,
        }
    }
}

struct OutputRecord {
    binding: DeliveryBinding,
    path: PathBuf,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct MediaDeliverySupervisor {
    egress_policy: Arc<RwLock<EgressPolicy>>,
    outputs: Arc<Mutex<HashMap<String, OutputRecord>>>,
    sequence: Arc<AtomicU64>,
    ffmpeg_program: Arc<PathBuf>,
}

impl MediaDeliverySupervisor {
    pub fn new(egress_policy: Arc<RwLock<EgressPolicy>>) -> Self {
        Self {
            egress_policy,
            outputs: Arc::new(Mutex::new(HashMap::new())),
            sequence: Arc::new(AtomicU64::new(1)),
            ffmpeg_program: Arc::new(PathBuf::from("ffmpeg")),
        }
    }

    pub async fn start(
        &self,
        request: DeliveryRequest,
        broker: Arc<dyn DeliveryInputBroker>,
        cancellation: DeliveryCancellation,
    ) -> Result<DeliveryResult, DeliveryError> {
        let now = unix_seconds();
        validate_request(&request, now)?;
        if !request.authority.matches(&request.binding) {
            return Err(DeliveryError::StaleGeneration);
        }

        let workspace = delivery_workspace();
        std::fs::create_dir_all(&workspace).map_err(|_| DeliveryError::CleanupFailed)?;
        let result = self
            .run_process(&request, broker, cancellation, &workspace)
            .await;
        match result {
            Ok(output_path) => {
                if !request.authority.matches(&request.binding) {
                    remove_workspace(&workspace);
                    return Err(DeliveryError::StaleGeneration);
                }
                let output_bytes = std::fs::metadata(&output_path)
                    .map_err(|_| DeliveryError::OutputUnavailable)?
                    .len();
                if output_bytes == 0 || output_bytes > request.max_output_bytes {
                    remove_workspace(&workspace);
                    return Err(DeliveryError::OutputLimitExceeded);
                }
                let token = format!(
                    "d{}-{}",
                    self.sequence.fetch_add(1, Ordering::Relaxed),
                    Uuid::new_v4().simple()
                );
                let gateway_path = format!(
                    "/media/delivery/{}/{}/{}/{}/{}/{}/{}",
                    token,
                    request.binding.session_id,
                    request.binding.item_id,
                    request.binding.item_revision,
                    request.binding.media_generation,
                    request.binding.display_generation,
                    request.binding.group_id
                );
                let mut outputs = self.outputs.lock().expect("delivery outputs poisoned");
                self.retain_live_outputs(&mut outputs);
                outputs.insert(
                    token,
                    OutputRecord {
                        binding: request.binding.clone(),
                        path: output_path,
                        expires_at: Instant::now() + request.output_ttl,
                    },
                );
                Ok(DeliveryResult {
                    contract_version: DELIVERY_CONTRACT_VERSION,
                    binding: request.binding,
                    gateway_path,
                    content_type: "video/mp4",
                    output_bytes,
                })
            }
            Err(error) => {
                remove_workspace(&workspace);
                Err(error)
            }
        }
    }

    async fn run_process(
        &self,
        request: &DeliveryRequest,
        broker: Arc<dyn DeliveryInputBroker>,
        cancellation: DeliveryCancellation,
        workspace: &Path,
    ) -> Result<PathBuf, DeliveryError> {
        let mut inputs = Vec::with_capacity(request.inputs.len());
        for input in &request.inputs {
            let track = request
                .shape
                .tracks
                .iter()
                .find(|track| track.id == input.track_id)
                .ok_or(DeliveryError::InputCapabilityRejected)?;
            let policy = self
                .egress_policy
                .read()
                .expect("egress policy poisoned")
                .clone();
            policy
                .validate(&input.resource.url, &input.resource.egress_scope)
                .await
                .map_err(|_| DeliveryError::EgressRejected)?;
            if cancellation.is_cancelled() {
                return Err(DeliveryError::Cancelled);
            }
            let acquired = broker.acquire(input, workspace)?;
            if acquired.path.as_os_str().is_empty() {
                return Err(DeliveryError::BrokerRejected);
            }
            inputs.push((track.kind, acquired.path));
        }

        let video = inputs
            .iter()
            .find(|(kind, _)| *kind == MediaTrackKind::Video)
            .map(|(_, path)| path)
            .ok_or(DeliveryError::IncompleteTrackGroup)?;
        let audio = inputs
            .iter()
            .find(|(kind, _)| *kind == MediaTrackKind::Audio)
            .map(|(_, path)| path)
            .ok_or(DeliveryError::IncompleteTrackGroup)?;
        let output = workspace.join("delivery.mp4");
        let max_size = request.max_output_bytes.to_string();
        let command = StructuredCommand::new(self.ffmpeg_program.as_os_str().to_os_string())
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-nostdin")
            .arg("-y")
            .arg("-i")
            .arg(video.as_os_str().to_os_string())
            .arg("-i")
            .arg(audio.as_os_str().to_os_string())
            .arg("-map")
            .arg("0:v:0")
            .arg("-map")
            .arg("1:a:0")
            .arg("-c:v")
            .arg("copy")
            .arg("-c:a")
            .arg("copy")
            .arg("-movflags")
            .arg("+faststart")
            .arg("-fs")
            .arg(max_size)
            .arg("-f")
            .arg("mp4")
            .arg(output.as_os_str().to_os_string());
        let mut child = command
            .into_tokio_command()
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|_| DeliveryError::ProcessFailed)?;
        let deadline = tokio::time::Instant::now() + request.timeout;
        let mut wait = Box::pin(child.wait());
        loop {
            enum ProcessEvent {
                Finished(std::io::Result<std::process::ExitStatus>),
                Cancelled,
                TimedOut,
                Tick,
            }
            let event = tokio::select! {
                status = &mut wait => ProcessEvent::Finished(status),
                _ = wait_for_cancel(cancellation.clone()) => ProcessEvent::Cancelled,
                _ = tokio::time::sleep_until(deadline) => ProcessEvent::TimedOut,
                _ = sleep(Duration::from_millis(10)) => ProcessEvent::Tick,
            };
            match event {
                ProcessEvent::Finished(status) => {
                    let status = status.map_err(|_| DeliveryError::ProcessFailed)?;
                    if !status.success() {
                        return Err(DeliveryError::ProcessFailed);
                    }
                    break;
                }
                ProcessEvent::Cancelled => {
                    drop(wait);
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    return Err(DeliveryError::Cancelled);
                }
                ProcessEvent::TimedOut => {
                    drop(wait);
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    return Err(DeliveryError::TimedOut);
                }
                ProcessEvent::Tick => {
                    if std::fs::metadata(&output)
                        .map(|metadata| metadata.len() > request.max_output_bytes)
                        .unwrap_or(false)
                    {
                        drop(wait);
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                        return Err(DeliveryError::OutputLimitExceeded);
                    }
                }
            }
        }
        if std::fs::metadata(&output)
            .map(|metadata| metadata.len() > request.max_output_bytes)
            .unwrap_or(false)
        {
            return Err(DeliveryError::OutputLimitExceeded);
        }
        Ok(output)
    }

    pub(crate) async fn serve(
        &self,
        token: &str,
        binding: &DeliveryBinding,
        method: Method,
        request_headers: &HeaderMap,
    ) -> Response {
        if method != Method::GET && method != Method::HEAD {
            return (
                StatusCode::METHOD_NOT_ALLOWED,
                DeliveryError::MethodNotAllowed.to_string(),
            )
                .into_response();
        }
        let record = {
            let mut outputs = self.outputs.lock().expect("delivery outputs poisoned");
            self.retain_live_outputs(&mut outputs);
            let Some(record) = outputs.get(token) else {
                return (
                    StatusCode::NOT_FOUND,
                    DeliveryError::OutputUnavailable.to_string(),
                )
                    .into_response();
            };
            if &record.binding != binding {
                return (
                    StatusCode::FORBIDDEN,
                    DeliveryError::CapabilityBindingMismatch.to_string(),
                )
                    .into_response();
            }
            record.path.clone()
        };
        let bytes = match tokio::fs::read(&record).await {
            Ok(bytes) => bytes,
            Err(_) => {
                return (
                    StatusCode::GONE,
                    DeliveryError::OutputUnavailable.to_string(),
                )
                    .into_response();
            }
        };
        ranged_response(method, request_headers, bytes)
    }

    pub fn revoke(&self, gateway_path: &str) -> bool {
        let Some(token) = gateway_path.split('/').nth(3) else {
            return false;
        };
        let mut outputs = self.outputs.lock().expect("delivery outputs poisoned");
        outputs.remove(token).is_some_and(|record| {
            remove_workspace(record.path.parent().unwrap_or_else(|| Path::new("/")));
            true
        })
    }

    fn retain_live_outputs(&self, outputs: &mut HashMap<String, OutputRecord>) {
        let now = Instant::now();
        outputs.retain(|_, record| {
            if record.expires_at > now {
                true
            } else {
                remove_workspace(record.path.parent().unwrap_or_else(|| Path::new("/")));
                false
            }
        });
    }
}

fn validate_request(request: &DeliveryRequest, now: u64) -> Result<(), DeliveryError> {
    let binding = &request.binding;
    if !valid_identifier(&binding.session_id)
        || !valid_identifier(&binding.item_id)
        || !valid_identifier(&binding.group_id)
        || binding.item_revision == 0
        || binding.display_generation == 0
        || request.shape.version != site_adapter_api::MEDIA_SHAPE_VERSION
        || request.shape.tracks.len() > site_adapter_api::MAX_MEDIA_TRACKS
        || request.timeout.is_zero()
        || request.timeout > MAX_DELIVERY_TIMEOUT
        || request.output_ttl.is_zero()
        || request.output_ttl > MAX_DELIVERY_TTL
        || request.max_output_bytes == 0
        || request.max_output_bytes > MAX_DELIVERY_OUTPUT_BYTES
    {
        return Err(DeliveryError::InvalidRequest);
    }
    if request.shape.tracks.len() != 2 || request.inputs.len() != 2 {
        return Err(DeliveryError::IncompleteTrackGroup);
    }
    let mut video = None;
    let mut audio = None;
    let mut ids = std::collections::HashSet::new();
    for track in &request.shape.tracks {
        if !ids.insert(track.id.clone())
            || !valid_identifier(&track.id)
            || track.group_id.as_deref() != Some(binding.group_id.as_str())
            || !track.group_id.as_deref().is_some_and(valid_group)
            || track.access_ref.is_none()
            || !matches!(
                track.protocol,
                StreamProtocol::HttpFile | StreamProtocol::Hls | StreamProtocol::Dash
            )
        {
            return Err(DeliveryError::UnsupportedShape);
        }
        if track.expires_at.is_none() {
            return Err(DeliveryError::UnsupportedShape);
        }
        if track.expires_at.is_some_and(|expiry| expiry <= now) {
            return Err(DeliveryError::ExpiredInput);
        }
        match track.kind {
            MediaTrackKind::Video if video.replace(track).is_some() => {
                return Err(DeliveryError::IncompleteTrackGroup);
            }
            MediaTrackKind::Audio if audio.replace(track).is_some() => {
                return Err(DeliveryError::IncompleteTrackGroup);
            }
            MediaTrackKind::Video | MediaTrackKind::Audio => {}
            MediaTrackKind::Muxed => return Err(DeliveryError::UnsupportedShape),
        }
    }
    if video.is_none() || audio.is_none() {
        return Err(DeliveryError::IncompleteTrackGroup);
    }
    for input in &request.inputs {
        let Some(track) = request
            .shape
            .tracks
            .iter()
            .find(|track| track.id == input.track_id)
        else {
            return Err(DeliveryError::InputCapabilityRejected);
        };
        if track.access_ref.as_deref() != Some(input.access_ref.as_str())
            || input.resource.protocol != track.protocol
        {
            return Err(DeliveryError::InputCapabilityRejected);
        }
        if !valid_upstream_url(&input.resource.url) {
            return Err(DeliveryError::InputCapabilityRejected);
        }
        if input.resource.public_headers.values().any(|value| {
            value
                .to_str()
                .is_ok_and(|value| value.chars().any(char::is_control))
        }) {
            return Err(DeliveryError::InputCapabilityRejected);
        }
        if input.resource.public_headers.len() > 32
            || input.resource.public_headers.iter().any(|(name, value)| {
                value.to_str().is_err()
                    || crate::security::is_secret_header(
                        name.as_str(),
                        value.to_str().unwrap_or(""),
                    )
            })
        {
            return Err(DeliveryError::SecretMaterial);
        }
        if input.resource.secret_headers.len() > 32 {
            return Err(DeliveryError::SecretMaterial);
        }
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_DELIVERY_ID_BYTES
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
}

fn valid_group(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_DELIVERY_GROUP_BYTES
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
        && !contains_secret_marker(value)
}

fn valid_opaque_ref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
        && !contains_secret_marker(value)
}

fn valid_upstream_url(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
}

fn contains_secret_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    ["cookie", "authorization", "bearer", "token", "signed-url"]
        .iter()
        .any(|marker| lower.contains(marker))
}

fn delivery_workspace() -> PathBuf {
    std::env::temp_dir().join(format!("gateway-delivery-{}", Uuid::new_v4().simple()))
}

fn remove_workspace(path: &Path) {
    let _ = std::fs::remove_dir_all(path);
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

async fn wait_for_cancel(cancellation: DeliveryCancellation) {
    while !cancellation.is_cancelled() {
        sleep(Duration::from_millis(5)).await;
    }
}

fn ranged_response(method: Method, headers: &HeaderMap, bytes: Vec<u8>) -> Response {
    let len = bytes.len();
    let (status, body, range) =
        if let Some(value) = headers.get(RANGE).and_then(|v| v.to_str().ok()) {
            let Some((start, end)) = parse_range(value, len) else {
                return (
                    StatusCode::RANGE_NOT_SATISFIABLE,
                    DeliveryError::InvalidRange.to_string(),
                )
                    .into_response();
            };
            (
                StatusCode::PARTIAL_CONTENT,
                bytes[start..=end].to_vec(),
                Some((start, end)),
            )
        } else {
            (StatusCode::OK, bytes, None)
        };
    let body_len = body.len();
    let mut response = Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "video/mp4")
        .header(CONTENT_LENGTH, body_len.to_string())
        .header(ACCEPT_RANGES, "bytes");
    if let Some((start, end)) = range {
        response = response.header(CONTENT_RANGE, format!("bytes {start}-{end}/{len}"));
    }
    response
        .body(if method == Method::HEAD {
            Body::empty()
        } else {
            Body::from(body)
        })
        .expect("delivery response")
}

fn parse_range(value: &str, len: usize) -> Option<(usize, usize)> {
    if len == 0 || value.contains(',') {
        return None;
    }
    let (start, end) = value.strip_prefix("bytes=")?.split_once('-')?;
    let start = start.parse::<usize>().ok()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{EgressDnsResolver, EgressResolutionFuture, EgressScope};
    use axum::body::to_bytes;
    use site_adapter_api::MediaTrack;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[derive(Debug)]
    struct PublicResolver;
    impl EgressDnsResolver for PublicResolver {
        fn resolve<'a>(&'a self, _host: &'a str, port: u16) -> EgressResolutionFuture<'a> {
            Box::pin(async move {
                Ok(vec![SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
                    port,
                )])
            })
        }
    }

    #[derive(Default)]
    struct Authority {
        current: Mutex<Option<DeliveryBinding>>,
    }
    impl DeliveryAuthority for Authority {
        fn matches(&self, binding: &DeliveryBinding) -> bool {
            self.current.lock().expect("authority poisoned").as_ref() == Some(binding)
        }
    }

    struct FileBroker {
        files: HashMap<String, PathBuf>,
    }
    impl DeliveryInputBroker for FileBroker {
        fn acquire(
            &self,
            capability: &DeliveryInputCapability,
            _workspace: &Path,
        ) -> Result<BrokerInput, DeliveryError> {
            self.files
                .get(&capability.access_ref)
                .cloned()
                .map(BrokerInput::from_server_owned_path)
                .transpose()?
                .ok_or(DeliveryError::BrokerRejected)
        }
    }

    fn policy() -> Arc<RwLock<EgressPolicy>> {
        Arc::new(RwLock::new(EgressPolicy::with_resolver(Arc::new(
            PublicResolver,
        ))))
    }

    fn request(
        authority: Arc<Authority>,
        video: PathBuf,
        audio: PathBuf,
        now: u64,
    ) -> (DeliveryRequest, Arc<FileBroker>) {
        let binding = DeliveryBinding::new("session-1", "item-1", 1, 4, 2, "group-1");
        *authority.current.lock().unwrap() = Some(binding.clone());
        let video_track = MediaTrack {
            id: "video-1".into(),
            kind: MediaTrackKind::Video,
            group_id: Some("group-1".into()),
            protocol: StreamProtocol::Dash,
            codec: Some("avc1".into()),
            container: Some("mp4".into()),
            mime_type: Some("video/mp4".into()),
            width: Some(64),
            height: Some(64),
            bitrate: Some(1000),
            language: None,
            access_ref: Some("video-ref".into()),
            expires_at: Some(now + 60),
        };
        let audio_track = MediaTrack {
            id: "audio-1".into(),
            kind: MediaTrackKind::Audio,
            group_id: Some("group-1".into()),
            protocol: StreamProtocol::Dash,
            codec: Some("mp4a".into()),
            container: Some("m4a".into()),
            mime_type: Some("audio/mp4".into()),
            width: None,
            height: None,
            bitrate: Some(1000),
            language: Some("und".into()),
            access_ref: Some("audio-ref".into()),
            expires_at: Some(now + 60),
        };
        let shape = MediaShapeV1 {
            version: site_adapter_api::MEDIA_SHAPE_VERSION,
            tracks: vec![video_track, audio_track],
        };
        let resource = |protocol| UpstreamResource {
            url: Url::parse("https://media.example.test/input").unwrap(),
            protocol,
            public_headers: HeaderMap::new(),
            secret_headers: HeaderMap::new(),
            egress_scope: EgressScope::PublicWeb,
        };
        let video_cap = DeliveryInputCapability::new_server_owned(
            "video-1",
            "video-ref",
            resource(StreamProtocol::Dash),
        )
        .unwrap();
        let audio_cap = DeliveryInputCapability::new_server_owned(
            "audio-1",
            "audio-ref",
            resource(StreamProtocol::Dash),
        )
        .unwrap();
        (
            DeliveryRequest {
                binding,
                shape,
                inputs: vec![video_cap, audio_cap],
                authority,
                timeout: DEFAULT_DELIVERY_TIMEOUT,
                output_ttl: DEFAULT_DELIVERY_TTL,
                max_output_bytes: MAX_DELIVERY_OUTPUT_BYTES,
            },
            Arc::new(FileBroker {
                files: HashMap::from([("video-ref".into(), video), ("audio-ref".into(), audio)]),
            }),
        )
    }

    #[test]
    fn validation_rejects_secret_refs_and_incomplete_groups() {
        assert!(
            DeliveryInputCapability::new_server_owned(
                "video-1",
                "cookie-ref",
                UpstreamResource {
                    url: Url::parse("https://media.example.test/input").unwrap(),
                    protocol: StreamProtocol::Dash,
                    public_headers: HeaderMap::new(),
                    secret_headers: HeaderMap::new(),
                    egress_scope: EgressScope::PublicWeb,
                },
            )
            .is_err()
        );
        let authority = Arc::new(Authority::default());
        let (mut request, _) = request(
            authority,
            PathBuf::from("video"),
            PathBuf::from("audio"),
            unix_seconds(),
        );
        request.shape.tracks.pop();
        assert_eq!(
            validate_request(&request, unix_seconds()),
            Err(DeliveryError::IncompleteTrackGroup)
        );
    }

    #[tokio::test]
    async fn stale_and_cancelled_attempts_do_not_publish_output() {
        let authority = Arc::new(Authority::default());
        let (request, broker) = request(
            authority.clone(),
            PathBuf::from("missing-video"),
            PathBuf::from("missing-audio"),
            unix_seconds(),
        );
        *authority.current.lock().unwrap() = None;
        let supervisor = MediaDeliverySupervisor::new(policy());
        let result = supervisor
            .start(request, broker, DeliveryCancellation::default())
            .await;
        assert_eq!(result, Err(DeliveryError::StaleGeneration));
        assert!(supervisor.outputs.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn expired_input_and_pre_cancel_are_bounded_before_ffmpeg() {
        let authority = Arc::new(Authority::default());
        let (mut request, broker) = request(
            authority.clone(),
            PathBuf::from("missing-video"),
            PathBuf::from("missing-audio"),
            0,
        );
        let supervisor = MediaDeliverySupervisor::new(policy());
        assert_eq!(
            supervisor
                .start(
                    request.clone(),
                    broker.clone(),
                    DeliveryCancellation::default(),
                )
                .await,
            Err(DeliveryError::ExpiredInput)
        );
        let now = unix_seconds();
        for track in &mut request.shape.tracks {
            track.expires_at = Some(now + 60);
        }
        let cancellation = DeliveryCancellation::default();
        cancellation.cancel();
        assert_eq!(
            supervisor.start(request, broker, cancellation).await,
            Err(DeliveryError::Cancelled)
        );
        assert!(supervisor.outputs.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn synthetic_fmp4_fixture_is_remuxed_and_projected() {
        let Some(video) = std::env::var_os("MEDIA_DELIVERY_FIXTURE_VIDEO") else {
            return;
        };
        let Some(audio) = std::env::var_os("MEDIA_DELIVERY_FIXTURE_AUDIO") else {
            return;
        };
        let authority = Arc::new(Authority::default());
        let (mut request, broker) = request(authority, video.into(), audio.into(), unix_seconds());
        request.timeout = Duration::from_secs(60);
        let supervisor = MediaDeliverySupervisor::new(policy());
        let cancellation = DeliveryCancellation::default();
        let result = supervisor
            .start(request.clone(), broker, cancellation)
            .await
            .expect("synthetic paired tracks should remux");
        assert_eq!(result.contract_version, DELIVERY_CONTRACT_VERSION);
        assert_eq!(result.content_type, "video/mp4");
        assert!(result.output_bytes > 0);
        assert!(result.output_bytes <= request.max_output_bytes);
        assert!(!result.gateway_path.contains("media.example.test"));
        assert!(!result.gateway_path.contains("video-ref"));
        let token = result.gateway_path.split('/').nth(3).unwrap();
        let response = supervisor
            .serve(token, &result.binding, Method::GET, &HeaderMap::new())
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[CONTENT_TYPE], "video/mp4");
        let bytes = to_bytes(response.into_body(), request.max_output_bytes as usize)
            .await
            .unwrap();
        assert_eq!(bytes.len() as u64, result.output_bytes);
        assert!(supervisor.revoke(&result.gateway_path));
    }

    #[tokio::test]
    async fn public_projection_is_bound_and_range_limited() {
        let supervisor = MediaDeliverySupervisor::new(policy());
        let binding = DeliveryBinding::new("s", "i", 1, 1, 1, "g");
        let token = "fixture".to_string();
        let dir = delivery_workspace();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("delivery.mp4");
        std::fs::write(&path, b"fixture-fmp4").unwrap();
        supervisor.outputs.lock().unwrap().insert(
            token.clone(),
            OutputRecord {
                binding: binding.clone(),
                path: path.clone(),
                expires_at: Instant::now() + Duration::from_secs(30),
            },
        );
        let response = supervisor
            .serve(
                &token,
                &binding,
                Method::GET,
                &HeaderMap::from_iter([(RANGE, "bytes=0-6".parse().unwrap())]),
            )
            .await;
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            to_bytes(response.into_body(), 1024).await.unwrap().as_ref(),
            b"fixture"
        );
        assert!(supervisor.revoke("/media/delivery/fixture/s/i/1/1/1/g"));
        assert!(!path.exists());
    }
}
