#![cfg(feature = "runtime-prep")]

use gateway_egress::R008Broker;
use generic_ytdlp::{
    BrokerProcessRunner, GenericYtdlpAdapter, ProcessError, RuntimeLimits, SafeBroker,
    render_blocked_summary, render_error_summary, render_success_summary,
};
use site_adapter_api::SiteAdapter;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(source) = args.next() else {
        return blocked("INVALID_ARGUMENTS", 64);
    };
    if args.next().is_some() {
        return blocked("INVALID_ARGUMENTS", 64);
    }

    // The smoke entrypoint owns the fixed interpreter selection. Callers may
    // provide only the prepared cache path; they cannot select a Python
    // executable for the worker.
    let python = PathBuf::from("python3");
    let Some(sandbox) = sandbox_path() else {
        return blocked("SANDBOX_UNAVAILABLE", 75);
    };
    let worker = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("worker/worker.py");
    if !worker.is_file() {
        return blocked("WORKER_UNAVAILABLE", 75);
    }

    // This is the only network authority in the smoke path. SafeBroker wraps
    // the real R008Broker only to retain bounded status/error classifications.
    let broker = SafeBroker::new(R008Broker::default());
    let pythonpath = env::var_os("YTDLP_PREP_PYTHONPATH").map(PathBuf::from);
    let limits = RuntimeLimits {
        pythonpath,
        ..RuntimeLimits::default()
    };
    let runner =
        BrokerProcessRunner::new(Arc::new(broker.clone()), python, worker, sandbox, limits);
    let adapter = GenericYtdlpAdapter::with_runtime_runner(Arc::new(runner));
    let recognized = match adapter.recognize(&source) {
        Ok(result) => result,
        Err(_) => return blocked("RECOGNITION_FAILED", 1),
    };
    let Some(locator) = recognized.locator.filter(|_| recognized.matched) else {
        let output = render_error_summary(
            &generic_ytdlp::YtdlpError::InvalidLocator,
            &broker.snapshot(),
        );
        print!("{output}");
        return ExitCode::from(2);
    };

    match adapter.resolve_detailed(&locator) {
        Ok(media) => {
            let output = render_success_summary(&media, &broker.snapshot());
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            let output = render_error_summary(&error, &broker.snapshot());
            let exit_code = match error {
                generic_ytdlp::YtdlpError::Parse(generic_ytdlp::ParseError::UnsupportedFormat)
                | generic_ytdlp::YtdlpError::Parse(
                    generic_ytdlp::ParseError::UnsupportedFormatStage(_),
                )
                | generic_ytdlp::YtdlpError::Parse(
                    generic_ytdlp::ParseError::UnsupportedProtocol,
                ) => 2,
                generic_ytdlp::YtdlpError::Process(ProcessError::Disabled) => 75,
                _ => 1,
            };
            print!("{output}");
            ExitCode::from(exit_code)
        }
    }
}

fn sandbox_path() -> Option<PathBuf> {
    let current = env::current_exe().ok()?;
    sandbox_path_for_executable(&current)
}

fn sandbox_path_for_executable(current: &Path) -> Option<PathBuf> {
    let candidate = current.parent()?.join("ytdlp-sandbox");
    let metadata = fs::symlink_metadata(&candidate).ok()?;
    (metadata.file_type().is_file() && metadata.permissions().mode() & 0o111 != 0)
        .then_some(candidate)
}

fn blocked(error_code: &'static str, exit_code: u8) -> ExitCode {
    let output = render_blocked_summary(
        error_code,
        &generic_ytdlp::BrokerDiagnosticsSnapshot::default(),
    );
    print!("{output}");
    ExitCode::from(exit_code)
}

#[cfg(test)]
mod tests {
    use super::sandbox_path_for_executable;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::PathBuf;

    fn fixture_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "generic-ytdlp-sandbox-binding-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn fixed_sibling_must_be_a_regular_executable_file() {
        let dir = fixture_dir("validation");
        let smoke = dir.join("generic-ytdlp-real-smoke");
        let sandbox = dir.join("ytdlp-sandbox");
        fs::write(&smoke, b"smoke").unwrap();

        assert_eq!(sandbox_path_for_executable(&smoke), None);

        fs::create_dir(&sandbox).unwrap();
        assert_eq!(sandbox_path_for_executable(&smoke), None);
        fs::remove_dir(&sandbox).unwrap();

        fs::write(&sandbox, b"sandbox").unwrap();
        fs::set_permissions(&sandbox, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(sandbox_path_for_executable(&smoke), None);

        fs::set_permissions(&sandbox, fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(sandbox_path_for_executable(&smoke), Some(sandbox.clone()));

        fs::remove_file(&sandbox).unwrap();
        let external = dir.join("external-sandbox");
        fs::write(&external, b"external").unwrap();
        fs::set_permissions(&external, fs::Permissions::from_mode(0o700)).unwrap();
        symlink(&external, &sandbox).unwrap();
        assert_eq!(sandbox_path_for_executable(&smoke), None);

        fs::remove_dir_all(dir).unwrap();
    }
}
