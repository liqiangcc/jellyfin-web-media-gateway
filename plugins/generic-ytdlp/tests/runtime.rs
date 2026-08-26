#![cfg(feature = "runtime-prep")]

use gateway_core::{EgressPolicy, EgressPolicyError, EgressScope};
use generic_ytdlp::{
    BrokerBackend, BrokerProcessRunner, BrokerRequest, BrokerResponse, R008Broker, RuntimeLimits,
    parse_machine_output,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

struct FixtureBroker;

impl BrokerBackend for FixtureBroker {
    fn handle(&self, request: BrokerRequest) -> BrokerResponse {
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

fn runner(timeout: Duration) -> BrokerProcessRunner {
    let pythonpath = std::env::var_os("YTDLP_SOURCE").map(PathBuf::from);
    let limits = RuntimeLimits {
        timeout,
        pythonpath,
        ..RuntimeLimits::default()
    };
    let sandbox = std::env::var_os("CARGO_BIN_EXE_ytdlp-sandbox")
        .map(PathBuf::from)
        .expect("cargo must provide the required sandbox binary");
    BrokerProcessRunner::new(
        Arc::new(FixtureBroker),
        PathBuf::from(std::env::var_os("PYTHON").unwrap_or_else(|| "python3".into())),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("worker/worker.py"),
        sandbox,
        limits,
    )
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
        "child_af_inet_denied",
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
}

#[test]
fn r008_broker_rejects_secret_userinfo_and_private_targets_before_network() {
    let broker = R008Broker::new(EgressPolicy::default(), Duration::from_secs(1));
    let response = broker.handle(BrokerRequest {
        operation: "http".into(),
        method: "GET".into(),
        url: "https://user:password@example.test/media".into(),
        headers: BTreeMap::new(),
        body: Vec::new(),
    });
    assert_eq!(response.error.as_deref(), Some("BROKER_URL_REJECTED"));
    let response = broker.handle(BrokerRequest {
        operation: "http".into(),
        method: "GET".into(),
        url: "http://127.0.0.1:9/metadata".into(),
        headers: BTreeMap::from([(String::from("Authorization"), String::from("Bearer secret"))]),
        body: Vec::new(),
    });
    assert_eq!(
        response.error.as_deref(),
        Some("BROKER_SECRET_HEADER_REJECTED")
    );
    let response = broker.handle(BrokerRequest {
        operation: "http".into(),
        method: "CONNECT".into(),
        url: "https://example.test/".into(),
        headers: BTreeMap::new(),
        body: Vec::new(),
    });
    assert_eq!(response.error.as_deref(), Some("BROKER_OPERATION_REJECTED"));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    assert_eq!(
        runtime.block_on(EgressPolicy::default().validate(
            &Url::parse("http://127.0.0.1/").unwrap(),
            &EgressScope::PublicWeb,
        )),
        Err(EgressPolicyError::TargetRejected)
    );
}
