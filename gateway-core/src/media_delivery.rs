//! Gateway-owned delivery for validated separated audio/video media.
//!
//! This module deliberately stops at the generic `MediaShapeV1` boundary. A
//! Site Plugin supplies only bounded track metadata and server-owned
//! capabilities. The broker turns those capabilities into controlled local
//! inputs; FFmpeg is given only those local inputs and structured arguments.
//! The browser receives a short-lived, session-bound Gateway path for the
//! resulting fMP4 file.

use crate::UpstreamResource;
use crate::control::ControlService;
use crate::security::{EgressPolicy, StructuredCommand, ValidatedTarget};
use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use serde::Serialize;
use site_adapter_api::{MediaShapeV1, MediaTrackKind, StreamProtocol};
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::time::Instant as TokioInstant;
use tokio::time::sleep;
use url::Url;
use uuid::Uuid;

pub const DELIVERY_CONTRACT_VERSION: u32 = 1;
pub const DEFAULT_DELIVERY_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_DELIVERY_TTL: Duration = Duration::from_secs(60);
pub const MAX_DELIVERY_TIMEOUT: Duration = Duration::from_secs(5 * 60);
pub const MAX_DELIVERY_TTL: Duration = Duration::from_secs(10 * 60);
pub const MAX_DELIVERY_OUTPUT_BYTES: u64 = 128 * 1024 * 1024;
pub const MAX_DELIVERY_INPUT_BYTES: u64 = 128 * 1024 * 1024;
pub const MAX_CONCURRENT_DELIVERIES: usize = 2;
pub const MAX_DELIVERY_OUTPUTS: usize = 32;
const MAX_DELIVERY_ID_BYTES: usize = 128;
const MAX_DELIVERY_GROUP_BYTES: usize = 128;
const MAX_DELIVERY_CONTAINER_BYTES: usize = 32;

/// The full CAS identity of one delivery attempt. No field is derived from a
/// short-lived URL or from browser-provided state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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

/// The result of broker acquisition. Only a local regular file is accepted;
/// URLs, manifests and segment references never cross this boundary into the
/// remux process. The broker must report metadata that matches the requested
/// track and the materialized file.
pub struct BrokerMaterialization {
    path: PathBuf,
    size_bytes: u64,
    protocol: StreamProtocol,
    container: String,
}

impl fmt::Debug for BrokerMaterialization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrokerMaterialization")
            .field("path", &"<broker-owned>")
            .field("size_bytes", &self.size_bytes)
            .field("protocol", &self.protocol)
            .field("container", &self.container)
            .finish()
    }
}

impl BrokerMaterialization {
    pub fn from_server_owned_path(
        path: PathBuf,
        size_bytes: u64,
        protocol: StreamProtocol,
        container: impl Into<String>,
    ) -> Result<Self, DeliveryError> {
        let container = container.into();
        if path.as_os_str().is_empty()
            || protocol != StreamProtocol::HttpFile
            || size_bytes == 0
            || size_bytes > MAX_DELIVERY_INPUT_BYTES
            || !valid_container(&container)
        {
            return Err(DeliveryError::BrokerRejected);
        }
        Ok(Self {
            path,
            size_bytes,
            protocol,
            container,
        })
    }
}

/// Egress validation and the checked DNS addresses are carried together with
/// the capability. A broker must use `pinned_client` for HTTP acquisition;
/// that client disables redirects and pins connections to the addresses that
/// EgressPolicy checked. The wrapper prevents an unbound URL from being the
/// acquisition input.
pub struct ValidatedDeliveryInput {
    capability: DeliveryInputCapability,
    target: ValidatedTarget,
    protocol: StreamProtocol,
    container: String,
}

impl fmt::Debug for ValidatedDeliveryInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidatedDeliveryInput")
            .field("track_id", &self.capability.track_id)
            .field("target", &self.target)
            .field("protocol", &self.protocol)
            .field("container", &self.container)
            .finish()
    }
}

impl ValidatedDeliveryInput {
    fn new(
        capability: DeliveryInputCapability,
        target: ValidatedTarget,
        protocol: StreamProtocol,
        container: String,
    ) -> Result<Self, DeliveryError> {
        if protocol != StreamProtocol::HttpFile
            || !valid_container(&container)
            || !target.is_bound_to(&capability.resource.url)
        {
            return Err(DeliveryError::EgressRejected);
        }
        Ok(Self {
            capability,
            target,
            protocol,
            container,
        })
    }

    pub fn track_id(&self) -> &str {
        &self.capability.track_id
    }

    pub fn access_ref(&self) -> &str {
        &self.capability.access_ref
    }

    pub fn url(&self) -> &Url {
        &self.capability.resource.url
    }

    pub fn public_headers(&self) -> &HeaderMap {
        &self.capability.resource.public_headers
    }

    pub fn secret_headers(&self) -> &HeaderMap {
        &self.capability.resource.secret_headers
    }

    pub fn target(&self) -> &ValidatedTarget {
        &self.target
    }

    pub fn pinned_client(
        &self,
        timeout: Option<Duration>,
    ) -> Result<reqwest::Client, reqwest::Error> {
        self.target.pinned_client_with_timeout(timeout)
    }

    pub fn protocol(&self) -> StreamProtocol {
        self.protocol
    }

    pub fn container(&self) -> &str {
        &self.container
    }
}

pub type DeliveryBrokerFuture<'a> =
    Pin<Box<dyn Future<Output = Result<BrokerMaterialization, DeliveryError>> + Send + 'a>>;

/// Compatibility name for callers that used the first-attempt broker type.
pub type BrokerInput = BrokerMaterialization;

/// This is the only input seam used by the delivery process. Acquisition is
/// async so cancellation/deadline can drop the download future. Production
/// brokers must use the validated input's no-redirect, pinned-DNS client and
/// materialize only a bounded local regular file.
pub trait DeliveryInputBroker: Send + Sync {
    fn acquire<'a>(
        &'a self,
        input: &'a ValidatedDeliveryInput,
        workspace: &'a Path,
    ) -> DeliveryBrokerFuture<'a>;
}

/// The production HTTP-file broker. It is deliberately only a broker of
/// already validated inputs: the supervisor performs EgressPolicy validation,
/// then this implementation uses the resulting pinned/no-redirect client and
/// keeps both public and secret headers on the server side.
#[derive(Clone, Debug)]
pub struct HttpFileDeliveryBroker {
    request_timeout: Duration,
}

impl Default for HttpFileDeliveryBroker {
    fn default() -> Self {
        Self {
            request_timeout: DEFAULT_DELIVERY_TIMEOUT,
        }
    }
}

impl HttpFileDeliveryBroker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(request_timeout: Duration) -> Self {
        Self { request_timeout }
    }
}

impl DeliveryInputBroker for HttpFileDeliveryBroker {
    fn acquire<'a>(
        &'a self,
        input: &'a ValidatedDeliveryInput,
        workspace: &'a Path,
    ) -> DeliveryBrokerFuture<'a> {
        Box::pin(async move {
            let request_timeout = self
                .request_timeout
                .min(MAX_DELIVERY_TIMEOUT)
                .max(Duration::from_millis(1));
            let client = input
                .pinned_client(Some(request_timeout))
                .map_err(|_| DeliveryError::BrokerRejected)?;
            let response = client
                .get(input.url().clone())
                .headers(input.public_headers().clone())
                .headers(input.secret_headers().clone())
                .send()
                .await
                .map_err(|_| DeliveryError::BrokerRejected)?;
            if !response.status().is_success() {
                return Err(DeliveryError::BrokerRejected);
            }
            if response
                .content_length()
                .is_some_and(|length| length > MAX_DELIVERY_INPUT_BYTES)
            {
                return Err(DeliveryError::InputLimitExceeded);
            }

            let path = workspace.join(format!("delivery-input-{}", input.track_id()));
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .await
                .map_err(|_| DeliveryError::BrokerRejected)?;
            let mut size_bytes = 0u64;
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|_| DeliveryError::BrokerRejected)?;
                size_bytes = size_bytes
                    .checked_add(chunk.len() as u64)
                    .ok_or(DeliveryError::InputLimitExceeded)?;
                if size_bytes > MAX_DELIVERY_INPUT_BYTES {
                    return Err(DeliveryError::InputLimitExceeded);
                }
                file.write_all(&chunk)
                    .await
                    .map_err(|_| DeliveryError::BrokerRejected)?;
            }
            file.flush()
                .await
                .map_err(|_| DeliveryError::BrokerRejected)?;
            drop(file);

            BrokerMaterialization::from_server_owned_path(
                path,
                size_bytes,
                input.protocol(),
                input.container(),
            )
        })
    }
}

/// The authority checks the PlaybackSession snapshot before and after the
/// external process. A stale result is discarded before it can be projected.
pub trait DeliveryAuthority: Send + Sync {
    fn matches(&self, binding: &DeliveryBinding) -> bool;
}

/// Adapter from the generic delivery supervisor to the Gateway's authoritative
/// PlaybackSession snapshot. It deliberately contains no site or URL logic.
#[derive(Clone)]
pub struct PlaybackDeliveryAuthority {
    control: ControlService,
    session_id: String,
}

impl PlaybackDeliveryAuthority {
    pub fn new(control: ControlService, session_id: impl Into<String>) -> Self {
        Self {
            control,
            session_id: session_id.into(),
        }
    }
}

impl DeliveryAuthority for PlaybackDeliveryAuthority {
    fn matches(&self, binding: &DeliveryBinding) -> bool {
        if binding.session_id != self.session_id {
            return false;
        }
        let Ok(snapshot) = self.control.snapshot(&self.session_id) else {
            return false;
        };
        snapshot.current_item.item_id == binding.item_id
            && snapshot.current_item.item_revision == binding.item_revision
            && snapshot.current_item.media_generation == binding.media_generation
            && snapshot.active_display.generation == binding.display_generation
    }
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
    InputLimitExceeded,
    ConcurrencyLimitExceeded,
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
            Self::InputLimitExceeded => "DELIVERY_INPUT_LIMIT",
            Self::ConcurrencyLimitExceeded => "DELIVERY_CONCURRENCY_LIMIT",
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
            | Self::InputLimitExceeded
            | Self::ConcurrencyLimitExceeded
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
    authority: Arc<dyn DeliveryAuthority>,
    path: PathBuf,
    expires_at: Instant,
    created_seq: u64,
}

#[derive(Clone)]
pub struct MediaDeliverySupervisor {
    egress_policy: Arc<RwLock<EgressPolicy>>,
    outputs: Arc<Mutex<HashMap<String, OutputRecord>>>,
    sequence: Arc<AtomicU64>,
    active_deliveries: Arc<AtomicU64>,
    ffmpeg_program: Arc<PathBuf>,
}

impl MediaDeliverySupervisor {
    pub fn new(egress_policy: Arc<RwLock<EgressPolicy>>) -> Self {
        Self {
            egress_policy,
            outputs: Arc::new(Mutex::new(HashMap::new())),
            sequence: Arc::new(AtomicU64::new(1)),
            active_deliveries: Arc::new(AtomicU64::new(0)),
            ffmpeg_program: Arc::new(PathBuf::from("ffmpeg")),
        }
    }

    pub async fn start(
        &self,
        request: DeliveryRequest,
        broker: Arc<dyn DeliveryInputBroker>,
        cancellation: DeliveryCancellation,
    ) -> Result<DeliveryResult, DeliveryError> {
        let deadline = tokio::time::Instant::now() + request.timeout;
        let now = unix_seconds();
        validate_request(&request, now)?;
        if cancellation.is_cancelled() {
            return Err(DeliveryError::Cancelled);
        }
        if !request.authority.matches(&request.binding) {
            return Err(DeliveryError::StaleGeneration);
        }
        let _permit = self.try_acquire_delivery()?;

        let workspace = delivery_workspace();
        if std::fs::create_dir_all(&workspace).is_err() {
            remove_workspace(&workspace);
            return Err(DeliveryError::CleanupFailed);
        }
        let result = self
            .run_process(&request, broker, cancellation.clone(), &workspace, deadline)
            .await;
        match result {
            Ok(output_path) => {
                if cancellation.is_cancelled() || !request.authority.matches(&request.binding) {
                    remove_workspace(&workspace);
                    return Err(if cancellation.is_cancelled() {
                        DeliveryError::Cancelled
                    } else {
                        DeliveryError::StaleGeneration
                    });
                }
                let output_bytes = match std::fs::metadata(&output_path) {
                    Ok(metadata) if metadata.is_file() => metadata.len(),
                    Ok(_) | Err(_) => {
                        remove_workspace(&workspace);
                        return Err(DeliveryError::OutputUnavailable);
                    }
                };
                if output_bytes == 0 || output_bytes > request.max_output_bytes {
                    remove_workspace(&workspace);
                    return Err(DeliveryError::OutputLimitExceeded);
                }
                if tokio::time::Instant::now() >= deadline {
                    remove_workspace(&workspace);
                    return Err(DeliveryError::TimedOut);
                }
                let created_seq = self.sequence.fetch_add(1, Ordering::Relaxed);
                let token = format!("d{}-{}", created_seq, Uuid::new_v4().simple());
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
                let mut outputs = match self.outputs.lock() {
                    Ok(outputs) => outputs,
                    Err(_) => {
                        remove_workspace(&workspace);
                        return Err(DeliveryError::CleanupFailed);
                    }
                };
                self.retain_live_outputs(&mut outputs);
                if cancellation.is_cancelled() || !request.authority.matches(&request.binding) {
                    remove_workspace(&workspace);
                    return Err(if cancellation.is_cancelled() {
                        DeliveryError::Cancelled
                    } else {
                        DeliveryError::StaleGeneration
                    });
                }
                let expires_at = Instant::now() + request.output_ttl;
                outputs.insert(
                    token.clone(),
                    OutputRecord {
                        binding: request.binding.clone(),
                        authority: request.authority.clone(),
                        path: output_path,
                        expires_at,
                        created_seq,
                    },
                );
                drop(outputs);
                self.schedule_output_expiry(token.clone(), expires_at);
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
        deadline: tokio::time::Instant,
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
            let target = await_delivery_stage(
                policy.validate_and_resolve(&input.resource.url, &input.resource.egress_scope),
                &cancellation,
                deadline,
            )
            .await?
            .map_err(|_| DeliveryError::EgressRejected)?;
            let validated = ValidatedDeliveryInput::new(
                input.clone(),
                target,
                track.protocol,
                track
                    .container
                    .clone()
                    .ok_or(DeliveryError::UnsupportedShape)?,
            )?;
            let acquired = await_delivery_stage(
                broker.acquire(&validated, workspace),
                &cancellation,
                deadline,
            )
            .await??;
            validate_materialization(&validated, &acquired, workspace)?;
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
        if tokio::time::Instant::now() >= deadline {
            return Err(DeliveryError::TimedOut);
        }
        let command = ffmpeg_command(
            self.ffmpeg_program.as_path(),
            video,
            audio,
            &output,
            request.max_output_bytes,
        );
        let mut child = command
            .into_tokio_command()
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|_| DeliveryError::ProcessFailed)?;
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
                    if cancellation.is_cancelled() {
                        return Err(DeliveryError::Cancelled);
                    }
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
        if cancellation.is_cancelled() {
            return Err(DeliveryError::Cancelled);
        }
        Ok(output)
    }

    fn try_acquire_delivery(&self) -> Result<DeliveryPermit, DeliveryError> {
        loop {
            let current = self.active_deliveries.load(Ordering::Acquire);
            if current >= MAX_CONCURRENT_DELIVERIES as u64 {
                return Err(DeliveryError::ConcurrencyLimitExceeded);
            }
            if self
                .active_deliveries
                .compare_exchange(current, current + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(DeliveryPermit {
                    active_deliveries: self.active_deliveries.clone(),
                });
            }
        }
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
                let record = outputs.remove(token).expect("delivery output present");
                remove_workspace(record.path.parent().unwrap_or_else(|| Path::new("/")));
                return (
                    StatusCode::FORBIDDEN,
                    DeliveryError::CapabilityBindingMismatch.to_string(),
                )
                    .into_response();
            }
            if !record.authority.matches(&record.binding) {
                let record = outputs.remove(token).expect("delivery output present");
                remove_workspace(record.path.parent().unwrap_or_else(|| Path::new("/")));
                return (StatusCode::GONE, DeliveryError::StaleGeneration.to_string())
                    .into_response();
            }
            (record.path.clone(), record.authority.clone())
        };
        if !record.1.matches(binding) {
            self.remove_output(token);
            return (StatusCode::GONE, DeliveryError::StaleGeneration.to_string()).into_response();
        }
        let bytes = match tokio::fs::read(&record.0).await {
            Ok(bytes) => bytes,
            Err(_) => {
                self.remove_output(token);
                return (
                    StatusCode::GONE,
                    DeliveryError::OutputUnavailable.to_string(),
                )
                    .into_response();
            }
        };
        if !record.1.matches(binding) {
            self.remove_output(token);
            return (StatusCode::GONE, DeliveryError::StaleGeneration.to_string()).into_response();
        }
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

    #[cfg(feature = "control-ui-harness")]
    pub fn published_output_count(&self) -> usize {
        self.outputs.lock().map(|outputs| outputs.len()).unwrap_or(0)
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
        while outputs.len() >= MAX_DELIVERY_OUTPUTS {
            let Some(token) = outputs
                .iter()
                .min_by_key(|(_, record)| record.created_seq)
                .map(|(token, _)| token.clone())
            else {
                break;
            };
            if let Some(record) = outputs.remove(&token) {
                remove_workspace(record.path.parent().unwrap_or_else(|| Path::new("/")));
            }
        }
    }

    fn schedule_output_expiry(&self, token: String, expires_at: Instant) {
        let outputs = Arc::downgrade(&self.outputs);
        tokio::spawn(async move {
            tokio::time::sleep_until(TokioInstant::from_std(expires_at)).await;
            let Some(outputs) = outputs.upgrade() else {
                return;
            };
            let Ok(mut outputs) = outputs.lock() else {
                return;
            };
            let expired = outputs
                .get(&token)
                .is_some_and(|record| record.expires_at <= Instant::now());
            if expired {
                if let Some(record) = outputs.remove(&token) {
                    remove_workspace(record.path.parent().unwrap_or_else(|| Path::new("/")));
                }
            }
        });
    }

    fn remove_output(&self, token: &str) {
        let mut outputs = self.outputs.lock().expect("delivery outputs poisoned");
        if let Some(record) = outputs.remove(token) {
            remove_workspace(record.path.parent().unwrap_or_else(|| Path::new("/")));
        }
    }
}

struct DeliveryPermit {
    active_deliveries: Arc<AtomicU64>,
}

impl Drop for DeliveryPermit {
    fn drop(&mut self) {
        self.active_deliveries.fetch_sub(1, Ordering::AcqRel);
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
            || track.protocol != StreamProtocol::HttpFile
            || !track.container.as_deref().is_some_and(valid_container)
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
    let mut input_ids = std::collections::HashSet::new();
    for input in &request.inputs {
        if !input_ids.insert(input.track_id.clone()) {
            return Err(DeliveryError::InputCapabilityRejected);
        }
        let Some(track) = request
            .shape
            .tracks
            .iter()
            .find(|track| track.id == input.track_id)
        else {
            return Err(DeliveryError::InputCapabilityRejected);
        };
        if track.access_ref.as_deref() != Some(input.access_ref.as_str())
            || input.resource.protocol != StreamProtocol::HttpFile
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

fn valid_container(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_DELIVERY_CONTAINER_BYTES
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
}

fn validate_materialization(
    input: &ValidatedDeliveryInput,
    materialization: &BrokerMaterialization,
    workspace: &Path,
) -> Result<(), DeliveryError> {
    if materialization.protocol != input.protocol
        || materialization.container != input.container
        || materialization.path.as_os_str().is_empty()
    {
        return Err(DeliveryError::BrokerRejected);
    }
    if materialization.size_bytes == 0 || materialization.size_bytes > MAX_DELIVERY_INPUT_BYTES {
        return Err(DeliveryError::InputLimitExceeded);
    }
    let metadata = std::fs::symlink_metadata(&materialization.path)
        .map_err(|_| DeliveryError::BrokerRejected)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(DeliveryError::BrokerRejected);
    }
    let canonical_workspace =
        std::fs::canonicalize(workspace).map_err(|_| DeliveryError::BrokerRejected)?;
    let canonical_path =
        std::fs::canonicalize(&materialization.path).map_err(|_| DeliveryError::BrokerRejected)?;
    if !canonical_path.starts_with(&canonical_workspace)
        || metadata.len() != materialization.size_bytes
    {
        return Err(DeliveryError::BrokerRejected);
    }
    validate_mp4_input(&canonical_path)?;
    Ok(())
}

fn validate_mp4_input(path: &Path) -> Result<(), DeliveryError> {
    let bytes = std::fs::read(path).map_err(|_| DeliveryError::BrokerRejected)?;
    if bytes.len() < 16 || &bytes[4..8] != b"ftyp" {
        return Err(DeliveryError::BrokerRejected);
    }
    let first_box_size = u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as usize;
    if first_box_size < 16 || first_box_size > bytes.len() {
        return Err(DeliveryError::BrokerRejected);
    }
    inspect_mp4_boxes(&bytes, 0, bytes.len(), 0)?;
    Ok(())
}

const MAX_MP4_BOX_DEPTH: usize = 8;

fn inspect_mp4_boxes(
    bytes: &[u8],
    mut offset: usize,
    end: usize,
    depth: usize,
) -> Result<(), DeliveryError> {
    if depth > MAX_MP4_BOX_DEPTH {
        return Err(DeliveryError::BrokerRejected);
    }
    while offset < end {
        if end - offset < 8 {
            return Err(DeliveryError::BrokerRejected);
        }
        let size = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeliveryError::BrokerRejected)?,
        ) as usize;
        let box_type = &bytes[offset + 4..offset + 8];
        let (header, box_size) = if size == 1 {
            if end - offset < 16 {
                return Err(DeliveryError::BrokerRejected);
            }
            let high = u32::from_be_bytes(
                bytes[offset + 8..offset + 12]
                    .try_into()
                    .map_err(|_| DeliveryError::BrokerRejected)?,
            );
            if high != 0 {
                return Err(DeliveryError::BrokerRejected);
            }
            (
                16,
                u32::from_be_bytes(
                    bytes[offset + 12..offset + 16]
                        .try_into()
                        .map_err(|_| DeliveryError::BrokerRejected)?,
                ) as usize,
            )
        } else {
            (8, size)
        };
        if box_size < header || box_size > end - offset {
            return Err(DeliveryError::BrokerRejected);
        }
        let payload_start = offset + header;
        let box_end = offset + box_size;
        if box_type == b"dref" {
            validate_data_reference_box(bytes, payload_start, box_end)?;
        } else if box_type == b"urn " || box_type == b"rdrf" {
            return Err(DeliveryError::BrokerRejected);
        } else if is_mp4_container(box_type) {
            let child_start = if box_type == b"meta" {
                payload_start
                    .checked_add(4)
                    .ok_or(DeliveryError::BrokerRejected)?
            } else {
                payload_start
            };
            if child_start > box_end {
                return Err(DeliveryError::BrokerRejected);
            }
            inspect_mp4_boxes(bytes, child_start, box_end, depth + 1)?;
        }
        offset = box_end;
    }
    Ok(())
}

fn validate_data_reference_box(
    bytes: &[u8],
    payload_start: usize,
    box_end: usize,
) -> Result<(), DeliveryError> {
    if box_end - payload_start < 8 {
        return Err(DeliveryError::BrokerRejected);
    }
    let entry_count = u32::from_be_bytes(
        bytes[payload_start + 4..payload_start + 8]
            .try_into()
            .map_err(|_| DeliveryError::BrokerRejected)?,
    ) as usize;
    let mut offset = payload_start + 8;
    for _ in 0..entry_count {
        if box_end - offset < 8 {
            return Err(DeliveryError::BrokerRejected);
        }
        let size = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeliveryError::BrokerRejected)?,
        ) as usize;
        let box_type = &bytes[offset + 4..offset + 8];
        if size < 12 || size > box_end - offset {
            return Err(DeliveryError::BrokerRejected);
        }
        if box_type != b"url " {
            return Err(DeliveryError::BrokerRejected);
        }
        let flags = u32::from_be_bytes(
            bytes[offset + 8..offset + 12]
                .try_into()
                .map_err(|_| DeliveryError::BrokerRejected)?,
        ) & 0x00ff_ffff;
        if flags & 1 == 0 {
            return Err(DeliveryError::BrokerRejected);
        }
        offset += size;
    }
    if offset != box_end {
        return Err(DeliveryError::BrokerRejected);
    }
    Ok(())
}

fn is_mp4_container(box_type: &[u8]) -> bool {
    matches!(
        box_type,
        b"moov"
            | b"trak"
            | b"mdia"
            | b"minf"
            | b"dinf"
            | b"stbl"
            | b"edts"
            | b"mvex"
            | b"moof"
            | b"traf"
            | b"mfra"
            | b"meta"
    )
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
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return;
    };
    if !name.starts_with("gateway-delivery-")
        || path.parent() != Some(std::env::temp_dir().as_path())
    {
        return;
    }
    let _ = std::fs::remove_dir_all(path);
}

fn ffmpeg_command(
    program: &Path,
    video: &Path,
    audio: &Path,
    output: &Path,
    max_output_bytes: u64,
) -> StructuredCommand {
    StructuredCommand::new(program.as_os_str().to_os_string())
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-nostdin")
        .arg("-y")
        .arg("-protocol_whitelist")
        .arg("file")
        .arg("-f")
        .arg("mp4")
        .arg("-i")
        .arg(video.as_os_str().to_os_string())
        .arg("-protocol_whitelist")
        .arg("file")
        .arg("-f")
        .arg("mp4")
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
        .arg("+frag_keyframe+empty_moov+default_base_moof")
        .arg("-fs")
        .arg(max_output_bytes.to_string())
        .arg("-f")
        .arg("mp4")
        .arg(output.as_os_str().to_os_string())
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

async fn await_delivery_stage<T, E, F>(
    future: F,
    cancellation: &DeliveryCancellation,
    deadline: tokio::time::Instant,
) -> Result<Result<T, E>, DeliveryError>
where
    F: Future<Output = Result<T, E>>,
{
    tokio::pin!(future);
    tokio::select! {
        result = &mut future => Ok(result),
        _ = wait_for_cancel(cancellation.clone()) => Err(DeliveryError::Cancelled),
        _ = tokio::time::sleep_until(deadline) => Err(DeliveryError::TimedOut),
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
        fn acquire<'a>(
            &'a self,
            input: &'a ValidatedDeliveryInput,
            workspace: &'a Path,
        ) -> DeliveryBrokerFuture<'a> {
            Box::pin(async move {
                let source = self
                    .files
                    .get(input.access_ref())
                    .cloned()
                    .ok_or(DeliveryError::BrokerRejected)?;
                let path = workspace.join(format!("test-input-{}", input.track_id()));
                std::fs::copy(source, &path).map_err(|_| DeliveryError::BrokerRejected)?;
                let size = std::fs::metadata(&path)
                    .map_err(|_| DeliveryError::BrokerRejected)?
                    .len();
                BrokerInput::from_server_owned_path(path, size, input.protocol(), input.container())
            })
        }
    }

    struct BlockingBroker;
    impl DeliveryInputBroker for BlockingBroker {
        fn acquire<'a>(
            &'a self,
            _input: &'a ValidatedDeliveryInput,
            _workspace: &'a Path,
        ) -> DeliveryBrokerFuture<'a> {
            Box::pin(std::future::pending::<
                Result<BrokerMaterialization, DeliveryError>,
            >())
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
            protocol: StreamProtocol::HttpFile,
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
            protocol: StreamProtocol::HttpFile,
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
            resource(StreamProtocol::HttpFile),
        )
        .unwrap();
        let audio_cap = DeliveryInputCapability::new_server_owned(
            "audio-1",
            "audio-ref",
            resource(StreamProtocol::HttpFile),
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
                    protocol: StreamProtocol::HttpFile,
                    public_headers: HeaderMap::new(),
                    secret_headers: HeaderMap::new(),
                    egress_scope: EgressScope::PublicWeb,
                },
            )
            .is_err()
        );
        let authority = Arc::new(Authority::default());
        let (mut incomplete_request, _) = request(
            authority,
            PathBuf::from("video"),
            PathBuf::from("audio"),
            unix_seconds(),
        );
        incomplete_request.shape.tracks.pop();
        assert_eq!(
            validate_request(&incomplete_request, unix_seconds()),
            Err(DeliveryError::IncompleteTrackGroup)
        );
        let (mut dash_request, _) = request(
            Arc::new(Authority::default()),
            PathBuf::from("video"),
            PathBuf::from("audio"),
            unix_seconds(),
        );
        dash_request.shape.tracks[0].protocol = StreamProtocol::Dash;
        assert_eq!(
            validate_request(&dash_request, unix_seconds()),
            Err(DeliveryError::UnsupportedShape)
        );
    }

    #[test]
    fn ffmpeg_argv_whitelists_only_local_files_before_inputs() {
        let command = ffmpeg_command(
            Path::new("ffmpeg"),
            Path::new("/workspace/video-input"),
            Path::new("/workspace/audio-input"),
            Path::new("/workspace/delivery.mp4"),
            MAX_DELIVERY_OUTPUT_BYTES,
        );
        let args: Vec<_> = command
            .argv()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let whitelists: Vec<_> = args
            .iter()
            .enumerate()
            .filter_map(|(index, arg)| (arg == "-protocol_whitelist").then_some(index))
            .collect();
        let inputs: Vec<_> = args
            .iter()
            .enumerate()
            .filter_map(|(index, arg)| (arg == "-i").then_some(index))
            .collect();
        assert_eq!(whitelists.len(), 2);
        assert_eq!(inputs.len(), 2);
        for (whitelist, input) in whitelists.into_iter().zip(inputs) {
            assert_eq!(args[whitelist + 1], "file");
            assert!(whitelist < input);
            assert_eq!(args[input - 2], "-f");
            assert_eq!(args[input - 1], "mp4");
        }
        assert!(args.iter().all(|arg| !arg.contains("://")));
    }

    #[tokio::test]
    async fn materialization_rejects_workspace_escape_symlinks_and_disguised_manifests() {
        let workspace = delivery_workspace();
        std::fs::create_dir_all(&workspace).unwrap();
        let outside = workspace
            .parent()
            .unwrap()
            .join(format!("delivery-outside-{}", Uuid::new_v4().simple()));
        std::fs::write(&outside, b"not an input").unwrap();
        let target_url = Url::parse("https://8.8.8.8/input").unwrap();
        let target = EgressPolicy::default()
            .validate_and_resolve(&target_url, &crate::security::EgressScope::PublicWeb)
            .await
            .unwrap();
        let input = ValidatedDeliveryInput::new(
            DeliveryInputCapability::new_server_owned(
                "video-1",
                "video-ref",
                UpstreamResource {
                    url: target_url,
                    protocol: StreamProtocol::HttpFile,
                    public_headers: HeaderMap::new(),
                    secret_headers: HeaderMap::new(),
                    egress_scope: EgressScope::PublicWeb,
                },
            )
            .unwrap(),
            target,
            StreamProtocol::HttpFile,
            "mp4".into(),
        )
        .unwrap();
        let escaped = BrokerInput::from_server_owned_path(
            outside.clone(),
            std::fs::metadata(&outside).unwrap().len(),
            StreamProtocol::HttpFile,
            "mp4",
        )
        .unwrap();
        assert_eq!(
            validate_materialization(&input, &escaped, &workspace),
            Err(DeliveryError::BrokerRejected)
        );

        let manifest = workspace.join("manifest.mp4");
        std::fs::write(&manifest, b"#EXTM3U\nhttps://outside.example/segment.mp4\n").unwrap();
        let materialization = BrokerInput::from_server_owned_path(
            manifest.clone(),
            std::fs::metadata(&manifest).unwrap().len(),
            StreamProtocol::HttpFile,
            "mp4",
        )
        .unwrap();
        assert_eq!(
            validate_materialization(&input, &materialization, &workspace),
            Err(DeliveryError::BrokerRejected)
        );

        let valid = workspace.join("valid.mp4");
        std::fs::write(&valid, b"\0\0\0\x18ftypisom\0\0\0\0isom").unwrap();
        #[cfg(unix)]
        {
            let link = workspace.join("link.mp4");
            std::os::unix::fs::symlink(&valid, &link).unwrap();
            let linked = BrokerInput::from_server_owned_path(
                link,
                std::fs::metadata(&valid).unwrap().len(),
                StreamProtocol::HttpFile,
                "mp4",
            )
            .unwrap();
            assert_eq!(
                validate_materialization(&input, &linked, &workspace),
                Err(DeliveryError::BrokerRejected)
            );
        }

        let self_contained = workspace.join("self-contained.mp4");
        std::fs::write(
            &self_contained,
            mp4_with_data_reference(1, b"https://inert.example"),
        )
        .unwrap();
        assert!(validate_mp4_input(&self_contained).is_ok());
        let external = workspace.join("external.mp4");
        std::fs::write(&external, mp4_with_data_reference(0, b"relative-media.m4a")).unwrap();
        assert_eq!(
            validate_mp4_input(&external),
            Err(DeliveryError::BrokerRejected)
        );
        let unknown = workspace.join("unknown-dref.mp4");
        let mut unknown_bytes = mp4_with_data_reference(1, b"self-contained");
        let marker = unknown_bytes
            .windows(4)
            .position(|window| window == b"url ")
            .unwrap();
        unknown_bytes[marker..marker + 4].copy_from_slice(b"xxxx");
        std::fs::write(&unknown, unknown_bytes).unwrap();
        assert_eq!(
            validate_mp4_input(&unknown),
            Err(DeliveryError::BrokerRejected)
        );
        remove_workspace(&workspace);
        let _ = std::fs::remove_file(outside);
    }

    fn mp4_with_data_reference(flags: u32, payload: &[u8]) -> Vec<u8> {
        fn box_bytes(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
            let mut bytes = Vec::with_capacity(8 + payload.len());
            bytes.extend_from_slice(&(8u32 + payload.len() as u32).to_be_bytes());
            bytes.extend_from_slice(kind);
            bytes.extend_from_slice(payload);
            bytes
        }
        let mut ftyp_payload = Vec::from(*b"isom");
        ftyp_payload.extend_from_slice(&0u32.to_be_bytes());
        ftyp_payload.extend_from_slice(b"isom");
        let ftyp = box_bytes(b"ftyp", &ftyp_payload);
        let mut url_payload = flags.to_be_bytes().to_vec();
        url_payload.extend_from_slice(payload);
        let url = box_bytes(b"url ", &url_payload);
        let mut dref_payload = 0u32.to_be_bytes().to_vec();
        dref_payload.extend_from_slice(&1u32.to_be_bytes());
        dref_payload.extend_from_slice(&url);
        let dref = box_bytes(b"dref", &dref_payload);
        let dinf = box_bytes(b"dinf", &dref);
        let moov = box_bytes(b"moov", &dinf);
        let mdat = box_bytes(b"mdat", b"https://inert.example");
        [ftyp, moov, mdat].concat()
    }

    #[tokio::test]
    async fn scheduled_expiry_removes_unvisited_output() {
        let supervisor = MediaDeliverySupervisor::new(policy());
        let binding = DeliveryBinding::new("s", "i", 1, 1, 1, "g");
        let authority = Arc::new(Authority {
            current: Mutex::new(Some(binding.clone())),
        });
        let dir = delivery_workspace();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("delivery.mp4");
        std::fs::write(&path, b"fixture-fmp4").unwrap();
        let token = "scheduled".to_string();
        let expires_at = Instant::now() + Duration::from_millis(20);
        supervisor.outputs.lock().unwrap().insert(
            token.clone(),
            OutputRecord {
                binding,
                authority,
                path: path.clone(),
                expires_at,
                created_seq: 1,
            },
        );
        supervisor.schedule_output_expiry(token, expires_at);

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!path.exists());
        assert!(supervisor.outputs.lock().unwrap().is_empty());
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
    async fn cancellation_drops_an_inflight_broker_acquisition() {
        let authority = Arc::new(Authority::default());
        let (request, _) = request(
            authority,
            PathBuf::from("missing-video"),
            PathBuf::from("missing-audio"),
            unix_seconds(),
        );
        let supervisor = MediaDeliverySupervisor::new(policy());
        let cancellation = DeliveryCancellation::default();
        let worker_cancellation = cancellation.clone();
        let worker = tokio::spawn({
            let supervisor = supervisor.clone();
            async move {
                supervisor
                    .start(request, Arc::new(BlockingBroker), worker_cancellation)
                    .await
            }
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        cancellation.cancel();
        assert_eq!(worker.await.unwrap(), Err(DeliveryError::Cancelled));
        assert_eq!(supervisor.active_deliveries.load(Ordering::Acquire), 0);
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
        let authority = Arc::new(Authority {
            current: Mutex::new(Some(binding.clone())),
        });
        let token = "fixture".to_string();
        let dir = delivery_workspace();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("delivery.mp4");
        std::fs::write(&path, b"fixture-fmp4").unwrap();
        supervisor.outputs.lock().unwrap().insert(
            token.clone(),
            OutputRecord {
                binding: binding.clone(),
                authority: authority.clone(),
                path: path.clone(),
                expires_at: Instant::now() + Duration::from_secs(30),
                created_seq: 1,
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
        *authority.current.lock().unwrap() = None;
        let stale = supervisor
            .serve(&token, &binding, Method::GET, &HeaderMap::new())
            .await;
        assert_eq!(stale.status(), StatusCode::GONE);
        assert!(!path.exists());
    }
}
