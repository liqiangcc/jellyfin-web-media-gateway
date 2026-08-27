use crate::{BrokerDiagnosticsSnapshot, ProcessError, YtdlpError};
use site_adapter_api::{ResolvedMedia, StreamProtocol};
use std::fmt::Write;

/// Render only bounded, machine-readable fields. In particular, neither the
/// source locator nor any resolved stream URL is accepted by this API.
pub fn render_success_summary(
    media: &ResolvedMedia,
    diagnostics: &BrokerDiagnosticsSnapshot,
) -> String {
    let protocol = media
        .streams
        .first()
        .map(|stream| match stream.protocol {
            StreamProtocol::HttpFile => "http-file",
            StreamProtocol::Hls => "hls",
        })
        .unwrap_or("n/a");
    let mut output = String::new();
    append_common(&mut output, "PASS", diagnostics);
    let _ = writeln!(output, "protocol: {protocol}");
    let _ = writeln!(output, "stream_count: {}", media.streams.len());
    let _ = writeln!(output, "title_length: {}", media.title.len());
    output
}

/// Convert all process/parser failures to fixed classifications. Raw process
/// diagnostics, source URLs and worker stderr never cross this boundary.
pub fn render_error_summary(error: &YtdlpError, diagnostics: &BrokerDiagnosticsSnapshot) -> String {
    let (result, error_code) = match error {
        YtdlpError::InvalidLocator => ("UNSUPPORTED", "INVALID_LOCATOR"),
        YtdlpError::Process(process_error) => match process_error {
            ProcessError::Disabled => ("BLOCKED", "RUNTIME_DISABLED"),
            _ => ("FAIL", process_error_code(*process_error)),
        },
        YtdlpError::Parse(parse_error) => {
            let (result, code) = match parse_error {
                crate::ParseError::UnsupportedFormat => ("UNSUPPORTED", "UNSUPPORTED_FORMAT"),
                crate::ParseError::UnsupportedProtocol => ("UNSUPPORTED", "UNSUPPORTED_PROTOCOL"),
                crate::ParseError::DrmUnsupported => ("UNSUPPORTED", "DRM_UNSUPPORTED"),
                crate::ParseError::UnsupportedProtection => {
                    ("UNSUPPORTED", "UNSUPPORTED_PROTECTION")
                }
                _ => ("FAIL", "PARSE_ERROR"),
            };
            (result, code)
        }
    };
    let mut output = String::new();
    append_common(&mut output, result, diagnostics);
    let _ = writeln!(output, "protocol: n/a");
    let _ = writeln!(output, "stream_count: 0");
    let _ = writeln!(output, "title_length: n/a");
    let _ = writeln!(output, "process_error: {error_code}");
    output
}

pub fn render_blocked_summary(
    error_code: &'static str,
    diagnostics: &BrokerDiagnosticsSnapshot,
) -> String {
    let mut output = String::new();
    append_common(&mut output, "BLOCKED", diagnostics);
    let _ = writeln!(output, "protocol: n/a");
    let _ = writeln!(output, "stream_count: 0");
    let _ = writeln!(output, "title_length: n/a");
    let _ = writeln!(output, "process_error: {error_code}");
    output
}

fn append_common(output: &mut String, result: &str, diagnostics: &BrokerDiagnosticsSnapshot) {
    let _ = writeln!(output, "result: {result}");
    let _ = writeln!(output, "plugin: generic-ytdlp");
    let _ = writeln!(output, "broker_status_class: {}", status_class(diagnostics));
    let _ = writeln!(
        output,
        "broker_error_code: {}",
        diagnostics.last_error_code.as_deref().unwrap_or("n/a")
    );
    let _ = writeln!(
        output,
        "broker_request_count: {}",
        diagnostics.request_count
    );
}

fn status_class(diagnostics: &BrokerDiagnosticsSnapshot) -> String {
    diagnostics
        .last_status_class
        .map(|class| format!("{class}xx"))
        .unwrap_or_else(|| "n/a".into())
}

fn process_error_code(error: ProcessError) -> &'static str {
    match error {
        ProcessError::InvalidRequest => "INVALID_REQUEST",
        ProcessError::InvalidLimits => "INVALID_LIMITS",
        ProcessError::SpawnFailed => "SPAWN_FAILED",
        ProcessError::IoFailure => "IO_FAILURE",
        ProcessError::Cancelled => "CANCELLED",
        ProcessError::TimedOut => "TIMEOUT",
        ProcessError::StdoutLimitExceeded => "STDOUT_LIMIT_EXCEEDED",
        ProcessError::StderrLimitExceeded => "STDERR_LIMIT_EXCEEDED",
        ProcessError::NonZeroExit => "NONZERO_EXIT",
        ProcessError::BrokerIo => "BROKER_IO",
        ProcessError::BrokerProtocol => "BROKER_PROTOCOL",
        ProcessError::Disabled => "RUNTIME_DISABLED",
    }
}
