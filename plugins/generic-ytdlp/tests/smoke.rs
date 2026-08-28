#![cfg(feature = "runtime-prep")]

use generic_ytdlp::{
    BrokerBackend, BrokerCancellation, BrokerDiagnosticsSnapshot, BrokerRequest, BrokerResponse,
    GenericYtdlpAdapter, ProcessOutput, ProcessRequest, ProcessRunner, SafeBroker,
    render_error_summary, render_success_summary,
};
use site_adapter_api::SiteAdapter;
use std::collections::BTreeMap;
use std::sync::Arc;
use url::Url;

struct SuccessRunner;

impl ProcessRunner for SuccessRunner {
    fn run(&self, _request: &ProcessRequest) -> Result<ProcessOutput, generic_ytdlp::ProcessError> {
        Ok(ProcessOutput {
            stdout: br#"{"title":"safe fixture","protection":"clear","streams":[{"id":"primary","protocol":"http-file","url":"https://cdn.example.test/video.mp4","public_headers":{},"upstream_access_ref":null}]}"#.to_vec(),
        })
    }
}

struct UnsupportedRunner;

impl ProcessRunner for UnsupportedRunner {
    fn run(&self, _request: &ProcessRequest) -> Result<ProcessOutput, generic_ytdlp::ProcessError> {
        Ok(ProcessOutput {
            stdout: br#"{"error":"UNSUPPORTED_FORMAT"}"#.to_vec(),
        })
    }
}

struct DiagnosticBackend;

impl BrokerBackend for DiagnosticBackend {
    fn handle(&self, _request: BrokerRequest, _cancel: BrokerCancellation) -> BrokerResponse {
        BrokerResponse {
            status: 413,
            reason: "Payload Too Large".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            error: Some("BROKER_RESPONSE_TOO_LARGE".into()),
        }
    }
}

#[test]
fn success_summary_contains_only_safe_bounded_fields() {
    let adapter = GenericYtdlpAdapter::with_runtime_runner(Arc::new(SuccessRunner));
    let locator = adapter
        .recognize("https://source.example/watch?id=source-secret")
        .unwrap()
        .locator
        .unwrap();
    let media = adapter.resolve_detailed(&locator).unwrap();
    let summary = render_success_summary(&media, &BrokerDiagnosticsSnapshot::default());

    assert!(summary.contains("result: PASS"));
    assert!(summary.contains("protocol: http-file"));
    assert!(summary.contains("stream_count: 1"));
    assert!(summary.contains("title_length: 12"));
    for sentinel in [
        "source.example",
        "source-secret",
        "cdn.example",
        "video.mp4",
        "Authorization",
        "Bearer",
    ] {
        assert!(!summary.contains(sentinel), "summary leaked {sentinel}");
    }
}

#[test]
fn unsupported_summary_is_bounded_and_machine_readable() {
    let adapter = GenericYtdlpAdapter::with_runtime_runner(Arc::new(UnsupportedRunner));
    let locator = adapter
        .recognize("https://source.example/watch?item=unsupported")
        .unwrap()
        .locator
        .unwrap();
    let error = adapter.resolve_detailed(&locator).unwrap_err();
    let summary = render_error_summary(&error, &BrokerDiagnosticsSnapshot::default());

    assert!(summary.contains("result: UNSUPPORTED"));
    assert!(summary.contains("process_error: UNSUPPORTED_FORMAT"));
    assert!(!summary.contains("unsupported_stage:"));
    assert!(!summary.contains("source.example"));
    assert!(!summary.contains("item=unsupported"));
}

#[test]
fn unsupported_stage_summary_is_fixed_and_bounded() {
    let error =
        generic_ytdlp::YtdlpError::Parse(generic_ytdlp::ParseError::UnsupportedFormatStage(
            generic_ytdlp::UnsupportedStage::FallbackDetail,
        ));
    let summary = render_error_summary(&error, &BrokerDiagnosticsSnapshot::default());

    assert!(summary.contains("result: UNSUPPORTED"));
    assert!(summary.contains("process_error: UNSUPPORTED_FORMAT"));
    assert!(summary.contains("unsupported_stage: FALLBACK_DETAIL"));
    assert!(summary.len() < 256);
    for sentinel in [
        "exception-sentinel",
        "https://fixture.invalid/?token=secret",
        "Authorization",
        "media-payload",
    ] {
        assert!(!summary.contains(sentinel));
    }
}

#[test]
fn unsupported_stage_reason_summary_is_fixed_and_bounded() {
    let error =
        generic_ytdlp::YtdlpError::Parse(generic_ytdlp::ParseError::UnsupportedFormatStageReason(
            generic_ytdlp::UnsupportedStage::FallbackPlayurl,
            generic_ytdlp::UnsupportedReason::PlayurlDashPresent,
        ));
    let summary = render_error_summary(&error, &BrokerDiagnosticsSnapshot::default());

    assert!(summary.contains("unsupported_stage: FALLBACK_PLAYURL"));
    assert!(summary.contains("fallback_reason: PLAYURL_DASH_PRESENT"));
    assert!(summary.len() < 320);
    for sentinel in [
        "exception-sentinel",
        "https://fixture.invalid/?token=secret",
        "Authorization",
        "media-payload",
    ] {
        assert!(!summary.contains(sentinel));
    }
}

#[test]
fn worker_failure_summaries_use_only_fixed_codes() {
    for (error, code) in [
        (
            generic_ytdlp::ParseError::RequestPolicyRejected,
            "REQUEST_POLICY_REJECTED",
        ),
        (generic_ytdlp::ParseError::BrokerFailure, "BROKER_FAILURE"),
        (
            generic_ytdlp::ParseError::ExtractorFailure,
            "EXTRACTOR_FAILURE",
        ),
        (
            generic_ytdlp::ParseError::UnsupportedFormat,
            "UNSUPPORTED_FORMAT",
        ),
        (
            generic_ytdlp::ParseError::UnexpectedWorkerFailure,
            "UNEXPECTED_WORKER_FAILURE",
        ),
    ] {
        let summary = render_error_summary(
            &generic_ytdlp::YtdlpError::Parse(error),
            &BrokerDiagnosticsSnapshot::default(),
        );
        assert!(summary.contains(&format!("process_error: {code}")));
        assert!(summary.len() < 256);
    }
}

#[test]
fn safe_broker_exposes_only_allowlisted_error_classification() {
    let broker = SafeBroker::new(DiagnosticBackend);
    let response = broker.handle(
        BrokerRequest {
            operation: "http".into(),
            method: "GET".into(),
            url: "https://source.example/watch?signed-secret=redacted".into(),
            headers: BTreeMap::new(),
            body: Vec::new(),
        },
        BrokerCancellation::default(),
    );
    assert_eq!(response.error.as_deref(), Some("BROKER_RESPONSE_TOO_LARGE"));
    assert_eq!(
        broker.snapshot(),
        BrokerDiagnosticsSnapshot {
            request_count: 1,
            last_status_class: Some(4),
            last_error_code: Some("BROKER_RESPONSE_TOO_LARGE".into()),
        }
    );
    let summary = render_error_summary(
        &generic_ytdlp::YtdlpError::Process(generic_ytdlp::ProcessError::BrokerProtocol),
        &broker.snapshot(),
    );
    assert!(summary.contains("broker_status_class: 4xx"));
    assert!(summary.contains("broker_error_code: BROKER_RESPONSE_TOO_LARGE"));
    assert!(!summary.contains("source.example"));
    assert!(!summary.contains("signed-secret"));
    let _ = Url::parse("https://source.example/watch").unwrap();
}
