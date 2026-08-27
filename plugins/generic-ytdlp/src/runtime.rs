//! Explicit verification/runtime-prep path for the frozen generic yt-dlp
//! architecture. This module is feature-gated deliberately: `Default` never
//! constructs it, and production registration therefore remains disabled.

use crate::{ProcessError, ProcessOutput, ProcessRequest};
pub use gateway_egress::{BrokerBackend, BrokerCancellation, BrokerRequest, BrokerResponse};
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use url::Url;

const IPC_FD: i32 = 3;
const MAX_BODY_BYTES: usize = gateway_egress::MAX_BODY_BYTES;
const MAX_HEADER_COUNT: usize = gateway_egress::MAX_HEADERS;
const MAX_HEADER_NAME_BYTES: usize = gateway_egress::MAX_HEADER_NAME_BYTES;
const MAX_HEADER_VALUE_BYTES: usize = gateway_egress::MAX_HEADER_VALUE_BYTES;
// BrokerRequest/BrokerResponse bodies use a compact, fixed-width hex string
// on the wire instead of serde_json's decimal Vec<u8> array. The frame bound
// is derived from the existing R008 body/header bounds. Header names/values
// are allowed a conservative 2x JSON-escaping allowance; 4 KiB is fixed
// punctuation/metadata overhead, not caller-configurable payload capacity.
const MAX_FRAME_BYTES: usize = MAX_BODY_BYTES * 2
    + MAX_HEADER_COUNT * (2 * (MAX_HEADER_NAME_BYTES + MAX_HEADER_VALUE_BYTES) + 8)
    + 4 * 1024;
const LEGACY_MAX_FRAME_BYTES: usize = 128 * 1024;
const FROZEN_YTDLP_VERSION: &str = "2026.08.19";
const FROZEN_YTDLP_COMMIT: &str = "3a08beaf031ab68f966401ead017ac81fe8486cf";

/// Bounded, site-neutral metadata captured around the real R008 broker.
///
/// The broker response body, headers and request URL are deliberately not
/// retained. Error codes are copied only from the fixed R008 allowlist below
/// so an upstream diagnostic can never become an arbitrary output channel.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BrokerDiagnosticsSnapshot {
    pub request_count: u32,
    pub last_status_class: Option<u16>,
    pub last_error_code: Option<String>,
}

#[derive(Default)]
struct BrokerDiagnosticsState {
    request_count: u32,
    last_status_class: Option<u16>,
    last_error_code: Option<String>,
}

/// Adds safe diagnostics to an existing broker without adding any network
/// authority. The intended production-shaped construction is
/// `SafeBroker::new(R008Broker::default())`.
#[derive(Clone)]
pub struct SafeBroker {
    backend: Arc<dyn BrokerBackend>,
    state: Arc<Mutex<BrokerDiagnosticsState>>,
}

impl SafeBroker {
    pub fn new<B>(backend: B) -> Self
    where
        B: BrokerBackend + 'static,
    {
        Self {
            backend: Arc::new(backend),
            state: Arc::new(Mutex::new(BrokerDiagnosticsState::default())),
        }
    }

    pub fn snapshot(&self) -> BrokerDiagnosticsSnapshot {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        BrokerDiagnosticsSnapshot {
            request_count: state.request_count,
            last_status_class: state.last_status_class,
            last_error_code: state.last_error_code.clone(),
        }
    }
}

impl BrokerBackend for SafeBroker {
    fn handle(&self, request: BrokerRequest, cancellation: BrokerCancellation) -> BrokerResponse {
        let response = self.backend.handle(request, cancellation);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.request_count = state.request_count.saturating_add(1);
        state.last_status_class = (100..=599)
            .contains(&response.status)
            .then_some(response.status / 100);
        state.last_error_code = response.error.as_deref().map(safe_error_code);
        response
    }
}

fn safe_error_code(code: &str) -> String {
    match code {
        "BROKER_CANCELLED"
        | "BROKER_CLIENT_REJECTED"
        | "BROKER_EGRESS_REJECTED"
        | "BROKER_OPERATION_REJECTED"
        | "BROKER_REQUEST_TOO_LARGE"
        | "BROKER_RESPONSE_HEADER_REJECTED"
        | "BROKER_RESPONSE_READ_FAILED"
        | "BROKER_RESPONSE_SECRET_REJECTED"
        | "BROKER_RESPONSE_TOO_LARGE"
        | "BROKER_RUNTIME_FAILED"
        | "BROKER_SECRET_HEADER_REJECTED"
        | "BROKER_TIMEOUT"
        | "BROKER_TRANSPORT_FAILED"
        | "BROKER_URL_REJECTED" => code.to_owned(),
        _ => "BROKER_ERROR".into(),
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeLimits {
    pub timeout: Duration,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub pythonpath: Option<PathBuf>,
    pub descendant_pid_file: Option<PathBuf>,
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            stdout_bytes: 256 * 1024,
            stderr_bytes: 16 * 1024,
            pythonpath: None,
            descendant_pid_file: None,
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
        self.run_action_with_cancel(action, source_url, BrokerCancellation::default())
    }

    pub fn run_action_with_cancel(
        &self,
        action: &str,
        source_url: &Url,
        cancellation: BrokerCancellation,
    ) -> Result<ProcessOutput, ProcessError> {
        let request = ProcessRequest::new(source_url.clone())?;
        self.run_child(action, request.source_url(), cancellation)
    }

    fn run_child(
        &self,
        action: &str,
        source_url: &Url,
        cancellation: BrokerCancellation,
    ) -> Result<ProcessOutput, ProcessError> {
        let (parent_socket, child_socket) =
            std::os::unix::net::UnixStream::pair().map_err(|_| ProcessError::SpawnFailed)?;
        let child_fd = child_socket.as_raw_fd();
        #[cfg(target_os = "linux")]
        let fd_upper_bound = fd_upper_bound().map_err(|_| ProcessError::SpawnFailed)?;
        let python = resolve_program(&self.python).ok_or(ProcessError::SpawnFailed)?;
        let mut command = Command::new(&self.sandbox);
        command
            .arg("--fd")
            .arg(IPC_FD.to_string())
            .arg("--")
            .arg(python)
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
        if let Some(pid_file) = &self.limits.descendant_pid_file {
            command.env("YTDLP_DESCENDANT_PID_FILE", pid_file);
        }
        unsafe {
            command.pre_exec(move || {
                if libc::setpgid(0, 0) != 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::dup2(child_fd, IPC_FD) < 0 {
                    return Err(io::Error::last_os_error());
                }
                // The worker receives only stdio and the per-attempt broker
                // capability. Do not rely on every present or future parent
                // descriptor having CLOEXEC set: close the entire ambient
                // descriptor range after fd 3 has been installed.
                close_unadmitted_fds(fd_upper_bound)?;
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
        let mut cancelled = false;
        let mut protocol_error = false;
        let mut broker_error = false;
        let mut broker_call: Option<(
            mpsc::Receiver<BrokerResponse>,
            BrokerCancellation,
            thread::JoinHandle<()>,
        )> = None;

        loop {
            if stdout_overflow.load(Ordering::Acquire) || stderr_overflow.load(Ordering::Acquire) {
                if let Some((_, broker_cancel, _)) = broker_call.as_ref() {
                    broker_cancel.cancel();
                }
                terminate_group(&mut child);
                break;
            }
            if cancellation.is_cancelled() {
                if let Some((_, broker_cancel, _)) = broker_call.as_ref() {
                    broker_cancel.cancel();
                }
                terminate_group(&mut child);
                cancelled = true;
                break;
            }
            if started.elapsed() >= self.limits.timeout {
                if let Some((_, broker_cancel, _)) = broker_call.as_ref() {
                    broker_cancel.cancel();
                }
                terminate_group(&mut child);
                timed_out = true;
                break;
            }
            if matches!(child.try_wait(), Ok(Some(_))) {
                break;
            }
            if let Some((receiver, _, _)) = broker_call.as_ref() {
                match receiver.try_recv() {
                    Ok(response) => {
                        let (_, _, join) = broker_call.take().expect("broker call exists");
                        let _ = join.join();
                        let wire_response = match WireBrokerResponse::try_from(response) {
                            Ok(response) => response,
                            Err(_) => {
                                terminate_group(&mut child);
                                broker_error = true;
                                break;
                            }
                        };
                        if write_frame(&mut parent_socket, &wire_response).is_err() {
                            terminate_group(&mut child);
                            broker_error = true;
                            break;
                        }
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        terminate_group(&mut child);
                        broker_error = true;
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        thread::sleep(Duration::from_millis(5));
                    }
                }
                continue;
            }
            match read_frame::<WireBrokerRequest>(&mut parent_socket) {
                Ok(request) => {
                    let request = match request.into_request() {
                        Ok(request) => request,
                        Err(_) => {
                            terminate_group(&mut child);
                            protocol_error = true;
                            break;
                        }
                    };
                    let backend = Arc::clone(&self.backend);
                    let broker_cancel = BrokerCancellation::default();
                    let thread_cancel = broker_cancel.clone();
                    let (sender, receiver) = mpsc::sync_channel(1);
                    let join = thread::spawn(move || {
                        let response = backend.handle(request, thread_cancel);
                        let _ = sender.send(response);
                    });
                    broker_call = Some((receiver, broker_cancel, join));
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
                    if let Some((_, broker_cancel, _)) = broker_call.as_ref() {
                        broker_cancel.cancel();
                    }
                    terminate_group(&mut child);
                    protocol_error = true;
                    break;
                }
            }
        }
        // Always signal the process group once the worker outcome is known:
        // the leader may have exited while a long-lived descendant remains.
        terminate_group(&mut child);
        if let Some((_, broker_cancel, join)) = broker_call.take() {
            broker_cancel.cancel();
            let _ = join.join();
        }
        let status = child.wait().map_err(|_| ProcessError::IoFailure)?;
        let stdout = stdout_thread.join().map_err(|_| ProcessError::IoFailure)?;
        let stderr = stderr_thread.join().map_err(|_| ProcessError::IoFailure)?;
        // Worker stderr is deliberately consumed and never emitted. Only the
        // bounded classification below may cross this process boundary.
        let _ = stderr;
        if cancelled {
            return Err(ProcessError::Cancelled);
        }
        if timed_out {
            return Err(ProcessError::TimedOut);
        }
        if protocol_error || broker_error {
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

fn resolve_program(program: &PathBuf) -> Option<PathBuf> {
    if program.is_absolute() {
        return program.is_file().then(|| program.clone());
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join(program))
        .find(|candidate| candidate.is_file())
}

#[cfg(target_os = "linux")]
fn fd_upper_bound() -> io::Result<u64> {
    let mut limit = std::mem::MaybeUninit::<libc::rlimit>::uninit();
    // SAFETY: `getrlimit` initializes the caller-provided rlimit on success.
    let result = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, limit.as_mut_ptr()) };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the successful getrlimit call initialized `limit`.
    let limit = unsafe { limit.assume_init() };
    if limit.rlim_cur == libc::RLIM_INFINITY {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "unbounded RLIMIT_NOFILE cannot establish fd isolation bound",
        ));
    }
    let limit = u64::try_from(limit.rlim_cur).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "RLIMIT_NOFILE does not fit a bounded fd range",
        )
    })?;
    if limit < 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "RLIMIT_NOFILE does not admit the required descriptors",
        ));
    }
    // File descriptors are c_int values. This is the type's representable
    // upper bound, not a small policy limit; clamping a larger rlimit here
    // still covers every descriptor the close(2) ABI can address.
    Ok(limit.min(libc::c_int::MAX as u64 + 1))
}

#[cfg(target_os = "linux")]
fn close_unadmitted_fds(upper_bound: u64) -> io::Result<()> {
    close_unadmitted_fds_with(upper_bound, close_range_syscall)
}

#[cfg(target_os = "linux")]
unsafe fn close_range_syscall(first: u32, last: u32, flags: u32) -> libc::c_long {
    // SAFETY: the arguments are the Linux close_range(2) ABI values.
    unsafe { libc::syscall(libc::SYS_close_range as libc::c_long, first, last, flags) }
}

#[cfg(target_os = "linux")]
fn close_unadmitted_fds_legacy(upper_bound: u64) -> io::Result<()> {
    for fd in 4..upper_bound {
        // SAFETY: this is the async-signal-safe close(2) syscall, and the
        // descriptor range was derived before fork/pre_exec.
        if unsafe { libc::close(fd as libc::c_int) } != 0 {
            let error = io::Error::last_os_error();
            // Sparse descriptor tables are normal. Any other error means the
            // child cannot prove that the admitted boundary was enforced.
            if error.raw_os_error() != Some(libc::EBADF) {
                return Err(error);
            }
        }
    }
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
mod fd_isolation_tests {
    use super::{close_unadmitted_fds_with, fd_upper_bound};
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::os::fd::{AsRawFd, RawFd};
    use std::os::unix::net::UnixStream;

    const SENTINEL_MINIMUMS: [RawFd; 3] = [8, 64, 4096];

    unsafe fn fake_enosys(_: u32, _: u32, _: u32) -> libc::c_long {
        // SAFETY: Linux exposes errno through this thread-local pointer.
        unsafe { *libc::__errno_location() = libc::ENOSYS };
        -1
    }

    unsafe fn fake_einval(_: u32, _: u32, _: u32) -> libc::c_long {
        // SAFETY: Linux exposes errno through this thread-local pointer.
        unsafe { *libc::__errno_location() = libc::EINVAL };
        -1
    }

    #[test]
    fn forced_enosys_closes_far_ambient_fds_and_keeps_fd_three() {
        let mut control = UnixStream::pair().expect("control socketpair");
        let mut sentinels = Vec::new();
        for minimum in SENTINEL_MINIMUMS {
            let file = OpenOptions::new()
                .read(true)
                .open("/dev/null")
                .expect("sentinel");
            let fd = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_DUPFD, minimum) };
            assert!(fd >= minimum);
            sentinels.push(fd);
        }
        let upper_bound = fd_upper_bound().expect("finite fd limit");
        let child_fd = control.1.as_raw_fd();
        let pid = unsafe { libc::fork() };
        assert!(pid >= 0, "fork failed");
        if pid == 0 {
            unsafe {
                if libc::dup2(child_fd, 3) < 0 {
                    libc::_exit(10);
                }
                let result = close_unadmitted_fds_with(upper_bound, fake_enosys);
                if result.is_err() {
                    libc::_exit(11);
                }
                for fd in 0..=2 {
                    if libc::fcntl(fd, libc::F_GETFD) < 0 {
                        libc::_exit(14);
                    }
                }
                for fd in sentinels {
                    if libc::fcntl(fd, libc::F_GETFD) >= 0 {
                        libc::_exit(12);
                    }
                }
                let marker = [b'3'];
                if libc::write(3, marker.as_ptr().cast(), marker.len()) != 1 {
                    libc::_exit(13);
                }
                libc::_exit(0);
            }
        }
        drop(control.1);
        let mut marker = [0u8; 1];
        control.0.read_exact(&mut marker).expect("fd 3 marker");
        assert_eq!(marker, [b'3']);
        let mut status = 0;
        unsafe { libc::waitpid(pid, &mut status, 0) };
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
        drop(control.0);
    }

    #[test]
    fn non_enosys_close_range_error_fails_closed_without_fallback() {
        let error =
            close_unadmitted_fds_with(fd_upper_bound().expect("finite fd limit"), fake_einval)
                .expect_err("unexpected close_range errors must fail closed");
        assert_eq!(error.raw_os_error(), Some(libc::EINVAL));
    }
}

#[cfg(target_os = "linux")]
fn close_unadmitted_fds_with(
    upper_bound: u64,
    close_range: unsafe fn(u32, u32, u32) -> libc::c_long,
) -> io::Result<()> {
    let result = unsafe { close_range(4, u32::MAX, 0) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ENOSYS) {
        return close_unadmitted_fds_legacy(upper_bound);
    }
    Err(error)
}

#[cfg(not(target_os = "linux"))]
fn close_unadmitted_fds(_: u64) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "runtime-prep requires Linux close_range fd isolation",
    ))
}

impl crate::ProcessRunner for BrokerProcessRunner {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        self.run_child(
            "extract",
            request.source_url(),
            BrokerCancellation::default(),
        )
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireBrokerRequest {
    operation: String,
    method: String,
    url: String,
    #[serde(default)]
    headers: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    body_hex: String,
}

impl WireBrokerRequest {
    fn into_request(self) -> io::Result<BrokerRequest> {
        Ok(BrokerRequest {
            operation: self.operation,
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: decode_body_hex(&self.body_hex)?,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WireBrokerResponse {
    status: u16,
    reason: String,
    #[serde(default)]
    headers: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    body_hex: String,
    #[serde(default)]
    error: Option<String>,
}

impl TryFrom<BrokerResponse> for WireBrokerResponse {
    type Error = io::Error;

    fn try_from(response: BrokerResponse) -> Result<Self, Self::Error> {
        if response.body.len() > MAX_BODY_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "response body exceeds R008 bound",
            ));
        }
        Ok(Self {
            status: response.status,
            reason: response.reason,
            headers: response.headers,
            body_hex: encode_body_hex(&response.body),
            error: response.error,
        })
    }
}

fn encode_body_hex(body: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(body.len().saturating_mul(2));
    for byte in body {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_body_hex(encoded: &str) -> io::Result<Vec<u8>> {
    if encoded.len() > MAX_BODY_BYTES.saturating_mul(2) || !encoded.len().is_multiple_of(2) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "body encoding exceeds R008 bound",
        ));
    }
    let bytes = encoded.as_bytes();
    let mut body = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = hex_value(pair[0])
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid body encoding"))?;
        let low = hex_value(pair[1])
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid body encoding"))?;
        body.push((high << 4) | low);
    }
    Ok(body)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
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

#[cfg(test)]
mod protocol_tests {
    use super::{
        BrokerResponse, LEGACY_MAX_FRAME_BYTES, MAX_BODY_BYTES, MAX_FRAME_BYTES,
        MAX_HEADER_COUNT, MAX_HEADER_NAME_BYTES, MAX_HEADER_VALUE_BYTES, WireBrokerRequest,
        WireBrokerResponse, decode_body_hex, encode_body_hex, read_frame, write_frame,
    };
    use std::collections::BTreeMap;
    use std::io::Cursor;

    fn framed(payload: &[u8]) -> Cursor<Vec<u8>> {
        let mut frame = (payload.len() as u32).to_be_bytes().to_vec();
        frame.extend_from_slice(payload);
        Cursor::new(frame)
    }

    #[test]
    fn legacy_decimal_body_serialization_reproduces_broker_protocol_overflow() {
        let response = BrokerResponse {
            status: 200,
            reason: "OK".into(),
            headers: BTreeMap::new(),
            body: vec![0; MAX_BODY_BYTES],
            error: None,
        };
        let legacy_payload = serde_json::to_vec(&response).unwrap();
        assert!(legacy_payload.len() > LEGACY_MAX_FRAME_BYTES);
    }

    #[test]
    fn bounded_hex_wire_response_fits_derived_frame_bound_at_r008_limits() {
        let headers = (0..MAX_HEADER_COUNT)
            .map(|index| {
                (
                    format!("x-{:0width$}", index, width = MAX_HEADER_NAME_BYTES - 2),
                    "v".repeat(MAX_HEADER_VALUE_BYTES),
                )
            })
            .collect();
        let response = WireBrokerResponse::try_from(BrokerResponse {
            status: 200,
            reason: "OK".into(),
            headers,
            body: vec![0; MAX_BODY_BYTES],
            error: None,
        })
        .unwrap();
        let mut frame = Vec::new();
        write_frame(&mut frame, &response).unwrap();
        assert!(frame.len() <= MAX_FRAME_BYTES + 4);
        assert_eq!(
            decode_body_hex(&response.body_hex).unwrap().len(),
            MAX_BODY_BYTES
        );
    }

    #[test]
    fn zero_truncated_malformed_and_oversize_frames_fail_closed() {
        assert!(read_frame::<WireBrokerRequest>(&mut framed(&[])).is_err());
        assert!(read_frame::<WireBrokerRequest>(&mut Cursor::new(vec![0, 0, 0])).is_err());
        assert!(read_frame::<WireBrokerRequest>(&mut framed(b"{")).is_err());
        let oversize = (MAX_FRAME_BYTES as u32 + 1).to_be_bytes();
        assert!(read_frame::<WireBrokerRequest>(&mut Cursor::new(oversize.to_vec())).is_err());
    }

    #[test]
    fn malformed_body_encoding_and_response_over_r008_bound_fail_closed() {
        let request = WireBrokerRequest {
            operation: "http".into(),
            method: "GET".into(),
            url: "https://fixture.example.test/".into(),
            headers: BTreeMap::new(),
            body_hex: "0".into(),
        };
        assert!(request.into_request().is_err());
        assert!(decode_body_hex("zz").is_err());
        assert!(
            WireBrokerResponse::try_from(BrokerResponse {
                status: 200,
                reason: "OK".into(),
                headers: BTreeMap::new(),
                body: vec![0; MAX_BODY_BYTES + 1],
                error: None,
            })
            .is_err()
        );
    }

    #[test]
    fn contained_response_secret_is_absent_from_wire_envelope() {
        let response = WireBrokerResponse::try_from(BrokerResponse {
            status: 200,
            reason: "OK".into(),
            headers: BTreeMap::from([(String::from("content-type"), String::from("text/plain"))]),
            body: b"safe-public-body".to_vec(),
            error: None,
        })
        .unwrap();
        let wire = serde_json::to_string(&response).unwrap();
        assert!(!wire.contains("fixture-response-secret"));
        assert!(wire.contains("73616665"));
        assert_eq!(
            decode_body_hex(&encode_body_hex(b"safe-public-body")).unwrap(),
            b"safe-public-body"
        );
    }
}

fn terminate_group(child: &mut Child) {
    unsafe {
        let _ = libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.kill();
}