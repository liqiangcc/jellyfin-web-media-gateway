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
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use url::Url;

const IPC_FD: i32 = 3;
const MAX_FRAME_BYTES: usize = 128 * 1024;
const FROZEN_YTDLP_VERSION: &str = "2026.08.19";
const FROZEN_YTDLP_COMMIT: &str = "3a08beaf031ab68f966401ead017ac81fe8486cf";

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
                close_unadmitted_fds()?;
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
                        if write_frame(&mut parent_socket, &response).is_err() {
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
            match read_frame::<BrokerRequest>(&mut parent_socket) {
                Ok(request) => {
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
fn close_unadmitted_fds() -> io::Result<()> {
    // fd 0..2 are the intentionally admitted stdio set and fd 3 is the
    // broker socketpair endpoint. close_range is atomic with respect to the
    // exec path and also closes the temporary child_socket descriptor.
    let result =
        unsafe { libc::syscall(libc::SYS_close_range as libc::c_long, 4u32, u32::MAX, 0u32) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(not(target_os = "linux"))]
fn close_unadmitted_fds() -> io::Result<()> {
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
