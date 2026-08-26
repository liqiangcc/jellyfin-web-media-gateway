#![cfg(feature = "runtime-prep")]

use gateway_egress::{BrokerCancellation as EgressCancellation, R008Broker};
use generic_ytdlp::{
    BrokerBackend, BrokerCancellation, BrokerProcessRunner, BrokerRequest, BrokerResponse,
    RuntimeLimits, parse_machine_output,
};
use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use url::Url;

struct FixtureBroker;

impl BrokerBackend for FixtureBroker {
    fn handle(&self, request: BrokerRequest, _cancellation: BrokerCancellation) -> BrokerResponse {
        assert_eq!(request.operation, "http");
        assert_eq!(request.method, "GET");
        assert!(request.headers.keys().all(|name| name != "Cookie"));
        BrokerResponse {
            status: 200,
            reason: "OK".into(),
            headers: BTreeMap::from([(
                String::from("content-type"),
                String::from("application/json"),
            )]),
            body: br#"{"fixture":"generic-ytdlp-broker","title":"fixture media"}"#.to_vec(),
            error: None,
        }
    }
}

fn runner_with_backend(
    backend: Arc<dyn BrokerBackend>,
    timeout: Duration,
    descendant_pid_file: Option<PathBuf>,
) -> BrokerProcessRunner {
    let pythonpath = std::env::var_os("YTDLP_SOURCE").map(PathBuf::from);
    let limits = RuntimeLimits {
        timeout,
        pythonpath,
        descendant_pid_file,
        ..RuntimeLimits::default()
    };
    let sandbox = std::env::var_os("CARGO_BIN_EXE_ytdlp-sandbox")
        .map(PathBuf::from)
        .expect("cargo must provide the required sandbox binary");
    BrokerProcessRunner::new(
        backend,
        PathBuf::from(std::env::var_os("PYTHON").unwrap_or_else(|| "python3".into())),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("worker/worker.py"),
        sandbox,
        limits,
    )
}

fn runner(timeout: Duration) -> BrokerProcessRunner {
    runner_with_backend(Arc::new(FixtureBroker), timeout, None)
}

fn pid_marker(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "generic-ytdlp-{label}-{}-{}.pid",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn marked_pid(path: &PathBuf) -> i32 {
    for _ in 0..400 {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(pid) = contents.trim().parse() {
                return pid;
            }
        }
        thread::sleep(Duration::from_millis(5));
    }
    panic!("descendant marker was not written: {}", path.display());
}

fn assert_pid_gone(pid: i32) {
    for _ in 0..200 {
        if !PathBuf::from(format!("/proc/{pid}")).exists() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("descendant {pid} survived process-group cleanup");
}

#[test]
fn pinned_worker_uses_actual_ytdlp_request_handler_and_existing_parser() {
    assert_eq!(
        BrokerProcessRunner::frozen_upstream(),
        ("2026.08.19", "3a08beaf031ab68f966401ead017ac81fe8486cf")
    );
    let output = runner(Duration::from_secs(5))
        .run_action(
            "probe",
            &Url::parse("https://fixture.example.test/media").unwrap(),
        )
        .unwrap();
    let media = parse_machine_output(&output.stdout).unwrap();
    assert_eq!(media.title, "fixture media");
    assert_eq!(media.streams.len(), 1);
}

#[test]
fn inherited_ipc_capability_supports_multiple_broker_requests() {
    let output = runner(Duration::from_secs(5))
        .run_action(
            "multi-probe",
            &Url::parse("https://fixture.example.test/media").unwrap(),
        )
        .unwrap();
    assert_eq!(
        parse_machine_output(&output.stdout).unwrap().title,
        "fixture media"
    );
}

#[test]
fn seccomp_denies_worker_custom_handler_and_child_but_ipc_survives() {
    let output = runner(Duration::from_secs(5))
        .run_action(
            "network-matrix",
            &Url::parse("https://fixture.example.test/media").unwrap(),
        )
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let matrix = &report["matrix"];
    for key in [
        "python_af_inet_denied",
        "python_af_inet6_denied",
        "custom_handler_denied",
        "custom_unix_handler_denied",
        "python_af_unix_denied",
        "child_af_inet_denied",
        "child_af_unix_denied",
        "broker_ipc_usable",
        "no_new_privs",
        "seccomp_filter",
    ] {
        assert_eq!(matrix[key], true, "{key}");
    }
}

#[test]
fn lifecycle_timeout_and_stdout_overflow_are_bounded() {
    let timeout_runner = runner(Duration::from_millis(300));
    assert_eq!(
        timeout_runner
            .run_action(
                "timeout",
                &Url::parse("https://fixture.example.test/media").unwrap()
            )
            .unwrap_err(),
        generic_ytdlp::ProcessError::TimedOut
    );
    let crash_runner = runner(Duration::from_secs(5));
    assert_eq!(
        crash_runner
            .run_action(
                "crash",
                &Url::parse("https://fixture.example.test/media").unwrap()
            )
            .unwrap_err(),
        generic_ytdlp::ProcessError::NonZeroExit
    );
    let overflow_runner = runner(Duration::from_secs(5));
    assert_eq!(
        overflow_runner
            .run_action(
                "overflow",
                &Url::parse("https://fixture.example.test/media").unwrap()
            )
            .unwrap_err(),
        generic_ytdlp::ProcessError::StdoutLimitExceeded
    );

    for (label, action, expected, timeout) in [
        (
            "timeout-descendant",
            "timeout-descendant",
            generic_ytdlp::ProcessError::TimedOut,
            Duration::from_millis(300),
        ),
        (
            "crash-descendant",
            "crash-descendant",
            generic_ytdlp::ProcessError::NonZeroExit,
            Duration::from_secs(5),
        ),
        (
            "overflow-descendant",
            "overflow-descendant",
            generic_ytdlp::ProcessError::StdoutLimitExceeded,
            Duration::from_secs(5),
        ),
    ] {
        let marker = pid_marker(label);
        let descendant_runner =
            runner_with_backend(Arc::new(FixtureBroker), timeout, Some(marker.clone()));
        assert_eq!(
            descendant_runner
                .run_action(
                    action,
                    &Url::parse("https://fixture.example.test/media").unwrap()
                )
                .unwrap_err(),
            expected
        );
        let pid = marked_pid(&marker);
        assert_pid_gone(pid);
        let _ = fs::remove_file(marker);
    }
}

#[test]
fn non_cloexec_ambient_fd_is_not_admitted_beyond_broker_fd() {
    let path = std::env::temp_dir().join(format!(
        "generic-ytdlp-ambient-{}-{}.sentinel",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let sentinel = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    unsafe {
        let fd = sentinel.as_raw_fd();
        let flags = libc::fcntl(fd, libc::F_GETFD);
        assert!(flags >= 0);
        assert_eq!(libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC), 0);
    }
    let output = runner(Duration::from_secs(5))
        .run_action(
            "ambient-fd",
            &Url::parse("https://fixture.example.test/media").unwrap(),
        )
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ambient_fds"].as_object().unwrap().len(), 0);
    assert_eq!(
        report["descendant_ambient_fds"].as_object().unwrap().len(),
        0
    );
    assert_eq!(report["broker_ipc_usable"], true);
    drop(sentinel);
    let _ = fs::remove_file(path);
}

struct BlockingBroker {
    started: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
}

impl BrokerBackend for BlockingBroker {
    fn handle(&self, _request: BrokerRequest, cancellation: BrokerCancellation) -> BrokerResponse {
        self.started.store(true, Ordering::Release);
        while !cancellation.is_cancelled() {
            thread::sleep(Duration::from_millis(5));
        }
        self.finished.store(true, Ordering::Release);
        BrokerResponse::denied("BROKER_CANCELLED")
    }
}

#[test]
fn external_cancel_wins_over_in_flight_broker_and_reaps_descendant() {
    let started = Arc::new(AtomicBool::new(false));
    let finished = Arc::new(AtomicBool::new(false));
    let backend = Arc::new(BlockingBroker {
        started: Arc::clone(&started),
        finished: Arc::clone(&finished),
    });
    let cancellation = BrokerCancellation::default();
    let marker = pid_marker("external-cancel");
    let runner = runner_with_backend(backend, Duration::from_secs(30), Some(marker.clone()));
    let cancel_for_thread = cancellation.clone();
    let task = thread::spawn(move || {
        runner.run_action_with_cancel(
            "cancel-probe-descendant",
            &Url::parse("https://fixture.example.test/media").unwrap(),
            cancel_for_thread,
        )
    });
    for _ in 0..400 {
        if started.load(Ordering::Acquire) {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(started.load(Ordering::Acquire));
    cancellation.cancel();
    let result = task.join().unwrap();
    assert_eq!(result.unwrap_err(), generic_ytdlp::ProcessError::Cancelled);
    assert!(finished.load(Ordering::Acquire));
    let pid = marked_pid(&marker);
    assert_pid_gone(pid);
    let _ = fs::remove_file(marker);
}

#[test]
fn diagnostics_consume_secret_sentinel_without_crossing_error_boundary() {
    let error = runner(Duration::from_secs(5))
        .run_action(
            "diagnostic-sentinel",
            &Url::parse("https://fixture.example.test/media").unwrap(),
        )
        .unwrap_err();
    assert_eq!(error, generic_ytdlp::ProcessError::NonZeroExit);
    let diagnostics = format!("{error:?}");
    for sentinel in ["signed-query-secret", "secret-token", "session-secret"] {
        assert!(!diagnostics.contains(sentinel));
    }
}

#[test]
fn r008_broker_rejects_secret_userinfo_and_private_targets_before_network() {
    let broker = R008Broker::default();
    let response = broker.handle(
        BrokerRequest {
            operation: "http".into(),
            method: "GET".into(),
            url: "https://user:password@example.test/media".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
        },
        EgressCancellation::default(),
    );
    assert_eq!(response.error.as_deref(), Some("BROKER_URL_REJECTED"));
    let response = broker.handle(
        BrokerRequest {
            operation: "http".into(),
            method: "GET".into(),
            url: "http://127.0.0.1:9/metadata".into(),
            headers: BTreeMap::from([(
                String::from("Authorization"),
                String::from("Bearer secret"),
            )]),
            body: Vec::new(),
        },
        EgressCancellation::default(),
    );
    assert_eq!(
        response.error.as_deref(),
        Some("BROKER_SECRET_HEADER_REJECTED")
    );
    let response = broker.handle(
        BrokerRequest {
            operation: "http".into(),
            method: "CONNECT".into(),
            url: "https://example.test/".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
        },
        EgressCancellation::default(),
    );
    assert_eq!(response.error.as_deref(), Some("BROKER_OPERATION_REJECTED"));
    let response = broker.handle(
        BrokerRequest {
            operation: "http".into(),
            method: "GET".into(),
            url: "http://127.0.0.1/".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
        },
        EgressCancellation::default(),
    );
    assert_eq!(response.error.as_deref(), Some("BROKER_EGRESS_REJECTED"));
}
