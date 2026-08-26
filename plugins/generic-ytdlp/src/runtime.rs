//! Explicit verification/runtime-prep path for the frozen generic yt-dlp
//! architecture. This module is feature-gated deliberately: `Default` never
//! constructs it, and production registration therefore remains disabled.

use crate::{ProcessError, ProcessOutput, ProcessRequest};
use futures_util::StreamExt;
use gateway_core::{EgressPolicy, EgressScope};
use serde::{Deserialize, Serialize};
use site_adapter_api::security::is_secret_header;
use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use url::Url;

const IPC_FD: i32 = 3;
const MAX_FRAME_BYTES: usize = 128 * 1024;
const MAX_BODY_BYTES: usize = 96 * 1024;
const MAX_HEADERS: usize = 32;
const MAX_HEADER_NAME_BYTES: usize = 128;
const MAX_HEADER_VALUE_BYTES: usize = 4096;
const FROZEN_YTDLP_VERSION: &str = "2026.08.19";
const FROZEN_YTDLP_COMMIT: &str = "3a08beaf031ab68f966401ead017ac81fe8486cf";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BrokerRequest {
    pub operation: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BrokerResponse {
    pub status: u16,
    pub reason: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Vec<u8>,
    #[serde(default)]
    pub error: Option<String>,
}

impl BrokerResponse {
    fn denied(code: &'static str) -> Self {
        Self {
            status: 400,
            reason: "Bad Request".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            error: Some(code.into()),
        }
    }
}

/// The only broker authority admitted to the worker. Test fixtures may
/// implement this trait, but production code must use `R008Broker`.
pub trait BrokerBackend: Send + Sync {
    fn handle(&self, request: BrokerRequest) -> BrokerResponse;
}

/// Gateway-owned HTTP(S) broker. Every request goes through the accepted
/// `EgressPolicy` resolver and the resulting checked address set is pinned in
/// the reqwest client. Redirects are intentionally returned, never followed.
pub struct R008Broker {
    policy: EgressPolicy,
    timeout: Duration,
}

impl R008Broker {
    pub fn new(policy: EgressPolicy, timeout: Duration) -> Self {
        Self { policy, timeout }
    }

    async fn handle_async(&self, request: BrokerRequest) -> BrokerResponse {
        if request.operation != "http" || !matches!(request.method.as_str(), "GET" | "HEAD") {
            return BrokerResponse::denied("BROKER_OPERATION_REJECTED");
        }
        if request.body.len() > MAX_BODY_BYTES || request.headers.len() > MAX_HEADERS {
            return BrokerResponse::denied("BROKER_REQUEST_TOO_LARGE");
        }
        for (name, value) in &request.headers {
            if name.is_empty()
                || name.len() > MAX_HEADER_NAME_BYTES
                || value.len() > MAX_HEADER_VALUE_BYTES
                || name.chars().any(char::is_control)
                || value.chars().any(char::is_control)
                || is_secret_header(name, value)
                || secretish_name(name)
            {
                return BrokerResponse::denied("BROKER_SECRET_HEADER_REJECTED");
            }
        }

        let Ok(url) = Url::parse(&request.url) else {
            return BrokerResponse::denied("BROKER_URL_REJECTED");
        };
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return BrokerResponse::denied("BROKER_URL_REJECTED");
        }

        let Ok(target) = self
            .policy
            .validate_and_resolve(&url, &EgressScope::PublicWeb)
            .await
        else {
            return BrokerResponse::denied("BROKER_EGRESS_REJECTED");
        };
        let Ok(client) = target.pinned_client_with_timeout(Some(self.timeout)) else {
            return BrokerResponse::denied("BROKER_CLIENT_REJECTED");
        };
        let method = match request.method.as_str() {
            "GET" => reqwest::Method::GET,
            "HEAD" => reqwest::Method::HEAD,
            _ => unreachable!(),
        };
        let mut builder = client.request(method, url);
        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }
        let Ok(response) = builder.body(request.body).send().await else {
            return BrokerResponse::denied("BROKER_TRANSPORT_FAILED");
        };
        let status = response.status();
        let reason = status.canonical_reason().unwrap_or("Unknown").to_string();
        let mut headers = BTreeMap::new();
        for (name, value) in response.headers() {
            let Ok(value) = value.to_str() else {
                return BrokerResponse::denied("BROKER_RESPONSE_HEADER_REJECTED");
            };
            if headers.len() >= MAX_HEADERS
                || name.as_str().len() > MAX_HEADER_NAME_BYTES
                || value.len() > MAX_HEADER_VALUE_BYTES
                || is_secret_header(name.as_str(), value)
            {
                return BrokerResponse::denied("BROKER_RESPONSE_SECRET_REJECTED");
            }
            headers.insert(name.as_str().into(), value.into());
        }
        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let Ok(chunk) = chunk else {
                return BrokerResponse::denied("BROKER_RESPONSE_READ_FAILED");
            };
            if body.len().saturating_add(chunk.len()) > MAX_BODY_BYTES {
                return BrokerResponse::denied("BROKER_RESPONSE_TOO_LARGE");
            }
            body.extend_from_slice(&chunk);
        }
        BrokerResponse {
            status: status.as_u16(),
            reason,
            headers,
            body,
            error: None,
        }
    }
}

impl BrokerBackend for R008Broker {
    fn handle(&self, request: BrokerRequest) -> BrokerResponse {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return BrokerResponse::denied("BROKER_RUNTIME_FAILED");
        };
        runtime.block_on(self.handle_async(request))
    }
}

fn secretish_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase().replace('_', "-");
    ["token", "credential", "password", "proxy-auth", "api-key"]
        .iter()
        .any(|needle| name.contains(needle))
}

#[derive(Clone, Debug)]
pub struct RuntimeLimits {
    pub timeout: Duration,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub pythonpath: Option<PathBuf>,
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            stdout_bytes: 256 * 1024,
            stderr_bytes: 16 * 1024,
            pythonpath: None,
        }
    }
}

pub struct BrokerProcessRunner {
    backend: Arc<dyn BrokerBackend>,
    python: PathBuf,
    worker: PathBuf,
    sandbox: PathBuf,
    limits: RuntimeLimits,
}

impl BrokerProcessRunner {
    pub fn new(
        backend: Arc<dyn BrokerBackend>,
        python: PathBuf,
        worker: PathBuf,
        sandbox: PathBuf,
        limits: RuntimeLimits,
    ) -> Self {
        Self {
            backend,
            python,
            worker,
            sandbox,
            limits,
        }
    }

    pub fn frozen_upstream() -> (&'static str, &'static str) {
        (FROZEN_YTDLP_VERSION, FROZEN_YTDLP_COMMIT)
    }

    pub fn run_action(
        &self,
        action: &str,
        source_url: &Url,
    ) -> Result<ProcessOutput, ProcessError> {
        let request = ProcessRequest::new(source_url.clone())?;
        self.run_child(action, request.source_url())
    }

    fn run_child(&self, action: &str, source_url: &Url) -> Result<ProcessOutput, ProcessError> {
        let (parent_socket, child_socket) =
            std::os::unix::net::UnixStream::pair().map_err(|_| ProcessError::SpawnFailed)?;
        let child_fd = child_socket.as_raw_fd();
        let mut command = Command::new(&self.sandbox);
        command
            .arg("--fd")
            .arg(IPC_FD.to_string())
            .arg("--")
            .arg(&self.python)
            .arg(&self.worker)
            .arg(action)
            .arg(source_url.as_str())
            .env_clear()
            .env("PYTHONNOUSERSITE", "1")
            .env("YTDLP_BROKER_FD", IPC_FD.to_string())
            .env("YTDLP_EXPECTED_VERSION", FROZEN_YTDLP_VERSION)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(pythonpath) = &self.limits.pythonpath {
            command.env("PYTHONPATH", pythonpath);
        }
        unsafe {
            command.pre_exec(move || {
                if libc::setpgid(0, 0) != 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::dup2(child_fd, IPC_FD) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = command.spawn().map_err(|_| ProcessError::SpawnFailed)?;
        drop(child_socket);
        let stdout = child.stdout.take().ok_or(ProcessError::IoFailure)?;
        let stderr = child.stderr.take().ok_or(ProcessError::IoFailure)?;
        let stdout_overflow = Arc::new(AtomicBool::new(false));
        let stderr_overflow = Arc::new(AtomicBool::new(false));
        let stdout_thread =
            capped_reader(stdout, self.limits.stdout_bytes, stdout_overflow.clone());
        let stderr_thread =
            capped_reader(stderr, self.limits.stderr_bytes, stderr_overflow.clone());
        let started = Instant::now();
        let mut parent_socket = parent_socket;
        parent_socket
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(|_| ProcessError::IoFailure)?;
        let mut timed_out = false;
        let mut protocol_error = false;

        loop {
            if stdout_overflow.load(Ordering::Acquire) || stderr_overflow.load(Ordering::Acquire) {
                terminate_group(&mut child);
                break;
            }
            if started.elapsed() >= self.limits.timeout {
                terminate_group(&mut child);
                timed_out = true;
                break;
            }
            if matches!(child.try_wait(), Ok(Some(_))) {
                break;
            }
            match read_frame::<BrokerRequest>(&mut parent_socket) {
                Ok(request) => {
                    let response = self.backend.handle(request);
                    write_frame(&mut parent_socket, &response)
                        .map_err(|_| ProcessError::BrokerIo)?;
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset
                    ) =>
                {
                    break;
                }
                Err(_) => {
                    terminate_group(&mut child);
                    protocol_error = true;
                    break;
                }
            }
        }
        if child.try_wait().ok().flatten().is_none() {
            terminate_group(&mut child);
        }
        let status = child.wait().map_err(|_| ProcessError::IoFailure)?;
        let stdout = stdout_thread.join().map_err(|_| ProcessError::IoFailure)?;
        let stderr = stderr_thread.join().map_err(|_| ProcessError::IoFailure)?;
        if std::env::var_os("YTDLP_DEBUG_RUNTIME").is_some() && !stderr.is_empty() {
            eprintln!(
                "worker stderr (redacted by contract): {} bytes",
                stderr.len()
            );
            eprintln!("{}", String::from_utf8_lossy(&stderr));
        }
        if timed_out {
            return Err(ProcessError::TimedOut);
        }
        if protocol_error {
            return Err(ProcessError::BrokerProtocol);
        }
        if stdout_overflow.load(Ordering::Acquire) {
            return Err(ProcessError::StdoutLimitExceeded);
        }
        if stderr_overflow.load(Ordering::Acquire) {
            return Err(ProcessError::StderrLimitExceeded);
        }
        if !status.success() {
            return Err(ProcessError::NonZeroExit);
        }
        Ok(ProcessOutput { stdout })
    }
}

impl crate::ProcessRunner for BrokerProcessRunner {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        self.run_child("probe", request.source_url())
    }
}

fn capped_reader<R: Read + Send + 'static>(
    mut reader: R,
    limit: usize,
    overflow: Arc<AtomicBool>,
) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut output = Vec::with_capacity(limit.min(8192));
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    if output.len().saturating_add(count) > limit {
                        overflow.store(true, Ordering::Release);
                        break;
                    }
                    output.extend_from_slice(&buffer[..count]);
                }
                Err(_) => {
                    overflow.store(true, Ordering::Release);
                    break;
                }
            }
        }
        output
    })
}

fn write_frame<T: Serialize>(stream: &mut impl Write, value: &T) -> io::Result<()> {
    let payload = serde_json::to_vec(value)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "frame serialization"))?;
    if payload.is_empty() || payload.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    stream.write_all(&(payload.len() as u32).to_be_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()
}

fn read_frame<T: for<'de> Deserialize<'de>>(stream: &mut impl Read) -> io::Result<T> {
    let mut length = [0u8; 4];
    stream.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload)?;
    serde_json::from_slice(&payload)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "frame decode"))
}

fn terminate_group(child: &mut Child) {
    unsafe {
        let _ = libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.kill();
}
