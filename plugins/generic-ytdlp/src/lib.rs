//! Deterministic preparation boundary for the generic yt-dlp SiteAdapter.
//!
//! The default adapter is deliberately disabled.  In particular, this crate
//! does not make a real network-capable yt-dlp executor reachable from the
//! production registry.  A later runtime task must put any such executor
//! behind the central EgressPolicy before it can be enabled.

use serde::Deserialize;
use site_adapter_api::{
    AdapterError, MediaProtection, RecognizeResult, ResolvedMedia, ResolvedStream, SiteAdapter,
    SiteAdapterRegistry, SourceLocator, StreamProtocol,
};
use std::collections::BTreeMap;
use std::fmt;
#[cfg(test)]
use std::io::{self, Read};
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(test)]
use std::thread;
#[cfg(test)]
use std::time::{Duration, Instant};
use url::Url;

pub const PLUGIN_ID: &str = "generic-ytdlp";
pub const SITE_ID: &str = "generic";
pub const LOCATOR_VERSION: u32 = 1;
pub const RECOGNITION_PRIORITY: u16 = 1;

#[cfg(feature = "runtime-prep")]
mod runtime;

#[cfg(feature = "runtime-prep")]
mod smoke;

#[cfg(feature = "runtime-prep")]
pub use runtime::{
    BrokerBackend, BrokerCancellation, BrokerDiagnosticsSnapshot, BrokerProcessRunner,
    BrokerRequest, BrokerResponse, RuntimeLimits, SafeBroker,
};

#[cfg(feature = "runtime-prep")]
pub use smoke::{render_blocked_summary, render_error_summary, render_success_summary};

const MAX_INPUT_BYTES: usize = 16 * 1024;
const MAX_JSON_BYTES: usize = 256 * 1024;
const MAX_TITLE_BYTES: usize = 1024;
const MAX_STREAMS: usize = 16;
const MAX_STREAM_ID_BYTES: usize = 128;
const MAX_URL_BYTES: usize = 16 * 1024;
const MAX_HEADERS: usize = 32;
const MAX_HEADER_NAME_BYTES: usize = 128;
const MAX_HEADER_VALUE_BYTES: usize = 4096;

/// Limits applied to a subprocess and each captured output stream.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessLimits {
    pub timeout: Duration,
    pub max_stdout_bytes: usize,
    pub max_stderr_bytes: usize,
}

#[cfg(test)]
impl Default for ProcessLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            max_stdout_bytes: MAX_JSON_BYTES,
            max_stderr_bytes: 16 * 1024,
        }
    }
}

#[cfg(test)]
impl ProcessLimits {
    fn valid(self) -> bool {
        !self.timeout.is_zero() && self.max_stdout_bytes > 0 && self.max_stderr_bytes > 0
    }
}

/// Structured input to the process boundary.  Callers cannot provide an
/// arbitrary argv or yt-dlp flag; the plugin owns the fixed command shape.
#[derive(Clone, Eq, PartialEq)]
pub struct ProcessRequest {
    source_url: Url,
}

impl fmt::Debug for ProcessRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProcessRequest")
            .field("source_url", &"<redacted>")
            .finish()
    }
}

impl ProcessRequest {
    pub fn new(source_url: Url) -> Result<Self, ProcessError> {
        if !matches!(source_url.scheme(), "http" | "https") || source_url.host_str().is_none() {
            return Err(ProcessError::InvalidRequest);
        }
        Ok(Self { source_url })
    }

    pub fn source_url(&self) -> &Url {
        &self.source_url
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessOutput {
    pub stdout: Vec<u8>,
}

/// Stable, bounded process classifications.  Raw stderr and executable
/// diagnostics never cross this boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessError {
    InvalidRequest,
    InvalidLimits,
    SpawnFailed,
    IoFailure,
    Cancelled,
    TimedOut,
    StdoutLimitExceeded,
    StderrLimitExceeded,
    NonZeroExit,
    BrokerIo,
    BrokerProtocol,
    Disabled,
}

pub trait ProcessRunner: Send + Sync {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError>;
}

/// A fixed-argv subprocess runner.  It is a reusable process boundary, not a
/// production network policy: the default SiteAdapter never constructs one.
#[cfg(test)]
struct CommandProcessRunner {
    executable: PathBuf,
    limits: ProcessLimits,
}

#[cfg(test)]
impl CommandProcessRunner {
    fn new(executable: impl Into<PathBuf>, limits: ProcessLimits) -> Result<Self, ProcessError> {
        if !limits.valid() {
            return Err(ProcessError::InvalidLimits);
        }
        Ok(Self {
            executable: executable.into(),
            limits,
        })
    }
}

#[cfg(test)]
impl ProcessRunner for CommandProcessRunner {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        let mut child = Command::new(&self.executable)
            .arg("--dump-single-json")
            .arg("--no-warnings")
            .arg("--no-playlist")
            .arg(request.source_url.as_str())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| ProcessError::SpawnFailed)?;

        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                terminate(&mut child);
                return Err(ProcessError::IoFailure);
            }
        };
        let stderr = match child.stderr.take() {
            Some(stderr) => stderr,
            None => {
                terminate(&mut child);
                return Err(ProcessError::IoFailure);
            }
        };
        let stdout_limit = self.limits.max_stdout_bytes;
        let stderr_limit = self.limits.max_stderr_bytes;
        let stdout_overflow_signal = Arc::new(AtomicBool::new(false));
        let stderr_overflow_signal = Arc::new(AtomicBool::new(false));
        let stdout_reader_signal = Arc::clone(&stdout_overflow_signal);
        let stderr_reader_signal = Arc::clone(&stderr_overflow_signal);
        let stdout_reader =
            thread::spawn(move || read_capped(stdout, stdout_limit, &stdout_reader_signal));
        let stderr_reader =
            thread::spawn(move || discard_capped(stderr, stderr_limit, &stderr_reader_signal));

        let started = Instant::now();
        let mut timed_out = false;
        let mut overflow_error = None;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if stdout_overflow_signal.load(Ordering::Acquire) => {
                    overflow_error = Some(ProcessError::StdoutLimitExceeded);
                    terminate(&mut child);
                    break;
                }
                Ok(None) if stderr_overflow_signal.load(Ordering::Acquire) => {
                    overflow_error = Some(ProcessError::StderrLimitExceeded);
                    terminate(&mut child);
                    break;
                }
                Ok(None) if started.elapsed() >= self.limits.timeout => {
                    timed_out = true;
                    terminate(&mut child);
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(2)),
                Err(_) => {
                    terminate(&mut child);
                    return Err(ProcessError::IoFailure);
                }
            }
        }

        let status = child.wait().map_err(|_| ProcessError::IoFailure)?;
        let (stdout, stdout_overflow) =
            stdout_reader.join().map_err(|_| ProcessError::IoFailure)?;
        let stderr_overflow = stderr_reader.join().map_err(|_| ProcessError::IoFailure)?;

        if timed_out {
            return Err(ProcessError::TimedOut);
        }
        if let Some(error) = overflow_error {
            return Err(error);
        }
        if stdout_overflow {
            return Err(ProcessError::StdoutLimitExceeded);
        }
        if stderr_overflow {
            return Err(ProcessError::StderrLimitExceeded);
        }
        if !status.success() {
            return Err(ProcessError::NonZeroExit);
        }
        Ok(ProcessOutput { stdout })
    }
}

#[cfg(test)]
fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
fn read_capped<R: Read>(
    mut reader: R,
    limit: usize,
    overflow_signal: &AtomicBool,
) -> (Vec<u8>, bool) {
    let mut output = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0_u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                let remaining = limit.saturating_sub(output.len());
                if remaining > 0 {
                    output.extend_from_slice(&buffer[..count.min(remaining)]);
                }
                if count > remaining {
                    overflow_signal.store(true, Ordering::Release);
                    return (output, true);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => {
                overflow_signal.store(true, Ordering::Release);
                return (output, true);
            }
        }
    }
    (output, false)
}

#[cfg(test)]
fn discard_capped<R: Read>(mut reader: R, limit: usize, overflow_signal: &AtomicBool) -> bool {
    let mut total = 0usize;
    let mut buffer = [0_u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return false,
            Ok(count) => {
                let remaining = limit.saturating_sub(total);
                if count > remaining {
                    overflow_signal.store(true, Ordering::Release);
                    return true;
                }
                total += count;
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => {
                overflow_signal.store(true, Ordering::Release);
                return true;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsupportedStage {
    PreFallback,
    FallbackWebpage,
    FallbackNav,
    FallbackView,
    FallbackDetail,
    FallbackPlayurl,
    MediaShape,
    Unclassified,
}

impl UnsupportedStage {
    pub fn from_wire(value: &str) -> Option<Self> {
        Some(match value {
            "PRE_FALLBACK" => Self::PreFallback,
            "FALLBACK_WEBPAGE" => Self::FallbackWebpage,
            "FALLBACK_NAV" => Self::FallbackNav,
            "FALLBACK_VIEW" => Self::FallbackView,
            "FALLBACK_DETAIL" => Self::FallbackDetail,
            "FALLBACK_PLAYURL" => Self::FallbackPlayurl,
            "MEDIA_SHAPE" => Self::MediaShape,
            "UNCLASSIFIED" => Self::Unclassified,
            _ => return None,
        })
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreFallback => "PRE_FALLBACK",
            Self::FallbackWebpage => "FALLBACK_WEBPAGE",
            Self::FallbackNav => "FALLBACK_NAV",
            Self::FallbackView => "FALLBACK_VIEW",
            Self::FallbackDetail => "FALLBACK_DETAIL",
            Self::FallbackPlayurl => "FALLBACK_PLAYURL",
            Self::MediaShape => "MEDIA_SHAPE",
            Self::Unclassified => "UNCLASSIFIED",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    Empty,
    Oversized,
    Malformed,
    UnsupportedSchema,
    InvalidField,
    InvalidUrl,
    UnsupportedProtocol,
    DrmUnsupported,
    UnsupportedProtection,
    SecretHeader,
    UnsupportedFormat,
    UnsupportedFormatStage(UnsupportedStage),
    RequestPolicyRejected,
    BrokerFailure,
    ExtractorFailure,
    UnexpectedWorkerFailure,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MachineOutput {
    title: String,
    streams: Vec<MachineStream>,
    protection: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MachineStream {
    id: String,
    protocol: String,
    url: String,
    #[serde(default)]
    public_headers: BTreeMap<String, String>,
    #[serde(default)]
    upstream_access_ref: Option<String>,
}

pub fn parse_machine_output(bytes: &[u8]) -> Result<ResolvedMedia, ParseError> {
    if bytes.is_empty() {
        return Err(ParseError::Empty);
    }
    if bytes.len() > MAX_JSON_BYTES {
        return Err(ParseError::Oversized);
    }
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| {
        if error.to_string().contains("unknown field") {
            ParseError::UnsupportedSchema
        } else if error.is_data() {
            ParseError::InvalidField
        } else {
            ParseError::Malformed
        }
    })?;
    if let Some(code) = value.get("error").and_then(serde_json::Value::as_str) {
        let Some(object) = value.as_object() else {
            return Err(ParseError::UnsupportedSchema);
        };
        if code != "UNSUPPORTED_FORMAT" && object.len() != 1 {
            return Err(ParseError::UnsupportedSchema);
        }
        return Err(match code {
            "REQUEST_POLICY_REJECTED" => ParseError::RequestPolicyRejected,
            "BROKER_FAILURE" => ParseError::BrokerFailure,
            "EXTRACTOR_FAILURE" => ParseError::ExtractorFailure,
            "UNSUPPORTED_FORMAT" => match object.get("unsupported_stage") {
                None if object.len() == 1 => ParseError::UnsupportedFormat,
                Some(serde_json::Value::String(stage)) if object.len() == 2 => {
                    UnsupportedStage::from_wire(stage)
                        .map(ParseError::UnsupportedFormatStage)
                        .unwrap_or(ParseError::UnsupportedSchema)
                }
                _ => ParseError::UnsupportedSchema,
            },
            "UNEXPECTED_WORKER_FAILURE" => ParseError::UnexpectedWorkerFailure,
            _ => ParseError::UnsupportedSchema,
        });
    }
    let output: MachineOutput = serde_json::from_value(value).map_err(|error| {
        if error.to_string().contains("unknown field") {
            ParseError::UnsupportedSchema
        } else if error.is_data() {
            ParseError::InvalidField
        } else {
            ParseError::Malformed
        }
    })?;
    if output.title.trim().is_empty() || output.title.len() > MAX_TITLE_BYTES {
        return Err(ParseError::InvalidField);
    }
    if output.protection != "clear" {
        return Err(match output.protection.as_str() {
            "drm_unsupported" => ParseError::DrmUnsupported,
            _ => ParseError::UnsupportedProtection,
        });
    }
    if output.streams.is_empty() || output.streams.len() > MAX_STREAMS {
        return Err(ParseError::InvalidField);
    }

    let mut streams = Vec::with_capacity(output.streams.len());
    for stream in output.streams {
        if stream.id.trim().is_empty() || stream.id.len() > MAX_STREAM_ID_BYTES {
            return Err(ParseError::InvalidField);
        }
        let protocol = match stream.protocol.as_str() {
            "http-file" => StreamProtocol::HttpFile,
            "hls" => StreamProtocol::Hls,
            _ => return Err(ParseError::UnsupportedProtocol),
        };
        if stream.url.len() > MAX_URL_BYTES {
            return Err(ParseError::InvalidField);
        }
        let url = Url::parse(&stream.url).map_err(|_| ParseError::InvalidUrl)?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(ParseError::InvalidUrl);
        }
        if stream.public_headers.len() > MAX_HEADERS {
            return Err(ParseError::InvalidField);
        }
        for (name, value) in &stream.public_headers {
            if name.is_empty()
                || name.len() > MAX_HEADER_NAME_BYTES
                || value.len() > MAX_HEADER_VALUE_BYTES
                || has_control(name)
                || has_control(value)
            {
                return Err(ParseError::InvalidField);
            }
            if site_adapter_api::security::is_secret_header(name, value) {
                return Err(ParseError::SecretHeader);
            }
        }
        if stream.upstream_access_ref.is_some() {
            return Err(ParseError::InvalidField);
        }
        streams.push(ResolvedStream {
            id: stream.id,
            protocol,
            url,
            public_headers: stream.public_headers,
            upstream_access_ref: None,
        });
    }
    Ok(ResolvedMedia {
        title: output.title,
        source_site: SITE_ID.into(),
        streams,
        subtitles: Vec::new(),
        protection: MediaProtection::Clear,
    })
}

fn has_control(value: &str) -> bool {
    value.chars().any(char::is_control)
}

struct DisabledRunner;

impl ProcessRunner for DisabledRunner {
    fn run(&self, _request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        Err(ProcessError::Disabled)
    }
}

/// Generic fallback adapter.  `Default` is the only adapter used by the
/// production registration helper and fails closed because its runner is
/// disabled.  Tests may inject a deterministic local runner explicitly.
pub struct GenericYtdlpAdapter {
    runner: Arc<dyn ProcessRunner>,
    runtime_enabled: bool,
}

impl Default for GenericYtdlpAdapter {
    fn default() -> Self {
        Self {
            runner: Arc::new(DisabledRunner),
            runtime_enabled: false,
        }
    }
}

impl GenericYtdlpAdapter {
    #[cfg(test)]
    fn with_runner(runner: Arc<dyn ProcessRunner>) -> Self {
        Self {
            runner,
            runtime_enabled: true,
        }
    }

    /// Construct the explicitly admitted verification/runtime adapter. The
    /// production registry continues to use `Default`, which is disabled.
    #[cfg(feature = "runtime-prep")]
    pub fn with_runtime_runner(runner: Arc<dyn ProcessRunner>) -> Self {
        Self {
            runner,
            runtime_enabled: true,
        }
    }

    pub fn is_runtime_enabled(&self) -> bool {
        self.runtime_enabled
    }

    pub fn resolve_detailed(&self, locator: &SourceLocator) -> Result<ResolvedMedia, YtdlpError> {
        validate_locator(locator)?;
        let url = Url::parse(&locator.opaque_payload).map_err(|_| YtdlpError::InvalidLocator)?;
        let request = ProcessRequest::new(url).map_err(YtdlpError::Process)?;
        let output = self.runner.run(&request).map_err(YtdlpError::Process)?;
        parse_machine_output(&output.stdout).map_err(YtdlpError::Parse)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum YtdlpError {
    InvalidLocator,
    Process(ProcessError),
    Parse(ParseError),
}

fn validate_locator(locator: &SourceLocator) -> Result<(), YtdlpError> {
    if locator.site_id != SITE_ID
        || locator.plugin_id != PLUGIN_ID
        || locator.locator_version != LOCATOR_VERSION
        || locator.opaque_payload.len() > MAX_INPUT_BYTES
    {
        return Err(YtdlpError::InvalidLocator);
    }
    Ok(())
}

impl SiteAdapter for GenericYtdlpAdapter {
    fn site_id(&self) -> &'static str {
        SITE_ID
    }

    fn plugin_id(&self) -> &'static str {
        PLUGIN_ID
    }

    fn recognize(&self, input: &str) -> Result<RecognizeResult, AdapterError> {
        let url = match Url::parse(input) {
            Ok(url) if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() => url,
            _ => return Ok(no_match()),
        };
        if input.len() > MAX_INPUT_BYTES
            || url.username() != ""
            || url.password().is_some()
            || is_direct_media(&url)
        {
            return Ok(no_match());
        }
        Ok(RecognizeResult {
            matched: true,
            site_id: SITE_ID.into(),
            plugin_id: PLUGIN_ID.into(),
            priority: RECOGNITION_PRIORITY,
            locator: Some(SourceLocator {
                site_id: SITE_ID.into(),
                plugin_id: PLUGIN_ID.into(),
                locator_version: LOCATOR_VERSION,
                opaque_payload: input.into(),
            }),
        })
    }

    fn resolve(&self, locator: &SourceLocator) -> Result<ResolvedMedia, AdapterError> {
        self.resolve_detailed(locator).map_err(|error| match error {
            YtdlpError::InvalidLocator => AdapterError::UnsupportedLocator,
            YtdlpError::Process(ProcessError::InvalidRequest) => AdapterError::InvalidInput,
            YtdlpError::Process(_) | YtdlpError::Parse(_) => AdapterError::InvalidAdapterOutput,
        })
    }
}

fn no_match() -> RecognizeResult {
    RecognizeResult {
        matched: false,
        site_id: SITE_ID.into(),
        plugin_id: PLUGIN_ID.into(),
        priority: RECOGNITION_PRIORITY,
        locator: None,
    }
}

fn is_direct_media(url: &Url) -> bool {
    let path = url.path().to_ascii_lowercase();
    path.ends_with(".mp4") || path.ends_with(".m4v") || path.ends_with(".m3u8")
}

/// Register only the fail-closed PREP adapter.  It is intentionally separate
/// from the real executor so registration order cannot enable networking.
pub fn register_prep_adapter(registry: &mut SiteAdapterRegistry) -> Result<(), AdapterError> {
    registry.register(Arc::new(GenericYtdlpAdapter::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use site_adapter_api::conformance::{
        RecognizeFixture, assert_adapter_conforms, assert_error_diagnostics_bounded,
    };

    #[derive(Clone)]
    struct FixtureRunner {
        output: Vec<u8>,
    }

    impl ProcessRunner for FixtureRunner {
        fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
            assert_eq!(request.source_url().scheme(), "https");
            Ok(ProcessOutput {
                stdout: self.output.clone(),
            })
        }
    }

    fn valid_json() -> Vec<u8> {
        br#"{"title":"fixture media","protection":"clear","streams":[{"id":"primary","protocol":"http-file","url":"https://cdn.example.test/video.mp4","public_headers":{"Accept":"video/mp4"},"upstream_access_ref":null}]}"#.to_vec()
    }

    fn adapter() -> GenericYtdlpAdapter {
        GenericYtdlpAdapter::with_runner(Arc::new(FixtureRunner {
            output: valid_json(),
        }))
    }

    fn locator(input: &str) -> SourceLocator {
        adapter().recognize(input).unwrap().locator.unwrap()
    }

    #[test]
    fn generic_ytdlp_conforms_with_deterministic_fixture() {
        assert_adapter_conforms(
            &adapter(),
            &[RecognizeFixture {
                input: "https://example.test/watch?id=fixture",
                expected_match: true,
                expected_site_id: SITE_ID,
                expected_locator_version: LOCATOR_VERSION,
            }],
        )
        .unwrap();
    }

    #[test]
    fn recognition_is_low_priority_and_does_not_steal_direct_media() {
        let generic = adapter();
        assert!(
            !generic
                .recognize("https://example.test/video.mp4")
                .unwrap()
                .matched
        );
        assert_eq!(
            generic
                .recognize("https://example.test/watch")
                .unwrap()
                .priority,
            1
        );
        assert!(
            !generic
                .recognize("ftp://example.test/watch")
                .unwrap()
                .matched
        );
    }

    #[test]
    fn default_runtime_is_disabled_and_fails_closed() {
        let adapter = GenericYtdlpAdapter::default();
        assert!(!adapter.is_runtime_enabled());
        assert_eq!(
            adapter.resolve(&locator("https://example.test/watch")),
            Err(AdapterError::InvalidAdapterOutput)
        );
    }

    #[test]
    fn registry_keeps_generic_direct_above_prep_fallback() {
        let mut registry = SiteAdapterRegistry::default();
        registry
            .register(Arc::new(generic_direct::GenericDirectAdapter))
            .unwrap();
        register_prep_adapter(&mut registry).unwrap();
        assert_eq!(
            registry
                .recognize("https://example.test/video.mp4")
                .unwrap()
                .plugin_id,
            "generic-direct"
        );
        assert_eq!(
            registry
                .recognize("https://example.test/watch")
                .unwrap()
                .plugin_id,
            PLUGIN_ID
        );
    }

    #[test]
    fn parser_rejects_malformed_unknown_drm_secret_and_unsupported_output() {
        assert_eq!(
            parse_machine_output(br"not-json"),
            Err(ParseError::Malformed)
        );
        assert_eq!(
            parse_machine_output(br#"{"title":"x","protection":"clear","streams":[],"extra":1}"#),
            Err(ParseError::UnsupportedSchema)
        );
        assert_eq!(parse_machine_output(br#"{"title":"x","protection":"drm_unsupported","streams":[{"id":"x","protocol":"hls","url":"https://cdn.example.test/x.m3u8"}]}"#), Err(ParseError::DrmUnsupported));
        assert_eq!(parse_machine_output(br#"{"title":"x","protection":"clear","streams":[{"id":"x","protocol":"hls","url":"https://cdn.example.test/x.m3u8","public_headers":{"Authorization":"Bearer secret"}}]}"#), Err(ParseError::SecretHeader));
        assert_eq!(parse_machine_output(br#"{"title":"x","protection":"clear","streams":[{"id":"x","protocol":"rtmp","url":"https://cdn.example.test/x"}]}"#), Err(ParseError::UnsupportedProtocol));
        assert_eq!(parse_machine_output(br#"{"title":"x","protection":"clear","streams":[{"id":"x","protocol":"hls","url":"file:///tmp/x"}]}"#), Err(ParseError::InvalidUrl));
        assert_eq!(parse_machine_output(br#"{"title":"x","protection":"clear","streams":[{"id":"x","protocol":"hls","url":"https://cdn.example.test/x.m3u8","upstream_access_ref":"forged-by-process"}]}"#), Err(ParseError::InvalidField));
        assert_eq!(
            parse_machine_output(br#"{"error":"NOT_AN_ADMITTED_FAILURE"}"#),
            Err(ParseError::UnsupportedSchema)
        );
        for stage in [
            UnsupportedStage::PreFallback,
            UnsupportedStage::FallbackWebpage,
            UnsupportedStage::FallbackNav,
            UnsupportedStage::FallbackView,
            UnsupportedStage::FallbackDetail,
            UnsupportedStage::FallbackPlayurl,
            UnsupportedStage::MediaShape,
            UnsupportedStage::Unclassified,
        ] {
            let envelope = format!(
                r#"{{"error":"UNSUPPORTED_FORMAT","unsupported_stage":"{}"}}"#,
                stage.as_str()
            );
            assert_eq!(
                parse_machine_output(envelope.as_bytes()),
                Err(ParseError::UnsupportedFormatStage(stage))
            );
        }
        for envelope in [
            br#"{"error":"UNSUPPORTED_FORMAT","unsupported_stage":"FORGED"}"#.as_slice(),
            br#"{"error":"UNSUPPORTED_FORMAT","unsupported_stage":null}"#.as_slice(),
            br#"{"error":"UNSUPPORTED_FORMAT","unsupported_stage":"MEDIA_SHAPE","message":"leak"}"#
                .as_slice(),
            br#"{"error":"EXTRACTOR_FAILURE","unsupported_stage":"MEDIA_SHAPE"}"#.as_slice(),
        ] {
            assert_eq!(
                parse_machine_output(envelope),
                Err(ParseError::UnsupportedSchema)
            );
        }
        assert_eq!(
            parse_machine_output(br#"{"error":"BROKER_FAILURE","message":"origin text"}"#),
            Err(ParseError::UnsupportedSchema)
        );
    }

    #[test]
    fn parser_maps_only_the_closed_worker_failure_taxonomy() {
        for (code, expected) in [
            ("REQUEST_POLICY_REJECTED", ParseError::RequestPolicyRejected),
            ("BROKER_FAILURE", ParseError::BrokerFailure),
            ("EXTRACTOR_FAILURE", ParseError::ExtractorFailure),
            ("UNSUPPORTED_FORMAT", ParseError::UnsupportedFormat),
            (
                "UNEXPECTED_WORKER_FAILURE",
                ParseError::UnexpectedWorkerFailure,
            ),
        ] {
            let envelope = format!(r#"{{"error":"{code}"}}"#);
            assert_eq!(parse_machine_output(envelope.as_bytes()), Err(expected));
        }
    }

    #[test]
    fn parser_maps_only_current_resolved_media_shape() {
        let media = parse_machine_output(&valid_json()).unwrap();
        assert_eq!(media.source_site, SITE_ID);
        assert_eq!(media.streams[0].protocol, StreamProtocol::HttpFile);
        assert_eq!(media.streams[0].public_headers["Accept"], "video/mp4");
        assert_eq!(media.protection, MediaProtection::Clear);
    }

    #[test]
    fn diagnostics_are_bounded_and_do_not_echo_secret_sentinels() {
        assert_error_diagnostics_bounded(&[
            "cookie-secret",
            "authorization-secret",
            "Bearer secret",
        ])
        .unwrap();
        for error in [
            ParseError::Malformed,
            ParseError::SecretHeader,
            ParseError::DrmUnsupported,
        ] {
            assert!(format!("{error:?}").len() < 64);
        }
    }

    #[test]
    fn process_request_debug_redacts_query_and_fragment() {
        let request = ProcessRequest::new(
            Url::parse("https://example.test/watch?token=secret-token#fragment-secret").unwrap(),
        )
        .unwrap();
        let diagnostics = format!("{request:?}");
        assert_eq!(diagnostics, "ProcessRequest { source_url: \"<redacted>\" }");
        assert!(!diagnostics.contains("secret-token"));
        assert!(!diagnostics.contains("fragment-secret"));
    }

    #[test]
    fn command_runner_uses_local_fake_argv_and_never_shell_interpolates() {
        let executable =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-ytdlp");
        let runner = CommandProcessRunner::new(
            executable,
            ProcessLimits {
                timeout: Duration::from_millis(100),
                max_stdout_bytes: 1024,
                max_stderr_bytes: 1024,
            },
        )
        .unwrap();
        let request = ProcessRequest::new(
            Url::parse("https://example.test/watch?mode=argv;touch%20/tmp/not-created").unwrap(),
        )
        .unwrap();
        let output = runner.run(&request).unwrap();
        assert_eq!(
            parse_machine_output(&output.stdout).unwrap().title,
            "fixture media"
        );
    }

    #[test]
    fn command_runner_bounds_timeout_stdout_stderr_and_exit_status() {
        let executable =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-ytdlp");
        let limits = ProcessLimits {
            timeout: Duration::from_millis(300),
            max_stdout_bytes: 512,
            max_stderr_bytes: 512,
        };
        let runner = CommandProcessRunner::new(executable, limits).unwrap();
        let run = |mode| {
            let request = ProcessRequest::new(
                Url::parse(&format!("https://example.test/watch?mode={mode}")).unwrap(),
            )
            .unwrap();
            runner.run(&request)
        };
        assert_eq!(run("timeout"), Err(ProcessError::TimedOut));
        let overflow_started = Instant::now();
        assert_eq!(
            run("stdout-overflow"),
            Err(ProcessError::StdoutLimitExceeded)
        );
        assert!(overflow_started.elapsed() < limits.timeout);
        let overflow_started = Instant::now();
        assert_eq!(
            run("stderr-overflow"),
            Err(ProcessError::StderrLimitExceeded)
        );
        assert!(overflow_started.elapsed() < limits.timeout);
        assert_eq!(run("nonzero"), Err(ProcessError::NonZeroExit));
    }
}
