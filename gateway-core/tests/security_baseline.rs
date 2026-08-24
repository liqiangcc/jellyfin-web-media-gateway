use gateway_core::security::{
    EgressPolicy, EgressPolicyError, EgressScope, SiteAccessCapability, SiteAccessError,
    is_public_web_ip, redact_text, redact_url,
};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;
use url::Url;

#[test]
fn public_web_matrix_covers_forbidden_ipv4_and_ipv6_classes() {
    let forbidden = [
        "0.0.0.0",
        "10.1.2.3",
        "100.64.0.1",
        "127.0.0.1",
        "169.254.169.254",
        "192.0.0.1",
        "192.0.2.1",
        "198.18.0.1",
        "224.0.0.1",
        "255.255.255.255",
        "::",
        "::1",
        "::ffff:127.0.0.1",
        "fc00::1",
        "fe80::1",
        "2001:db8::1",
        "ff02::1",
    ];
    for value in forbidden {
        assert!(
            !is_public_web_ip(value.parse::<IpAddr>().unwrap()),
            "{value}"
        );
    }
    for value in ["8.8.8.8", "1.1.1.1", "2001:4860:4860::8888"] {
        assert!(
            is_public_web_ip(value.parse::<IpAddr>().unwrap()),
            "{value}"
        );
    }
}

#[tokio::test]
async fn local_service_requires_named_configuration_and_rechecks_redirect_origin() {
    let mut policy = EgressPolicy::default();
    let origin = Url::parse("http://127.0.0.1:8096/").unwrap();
    policy.configure_local_service("jellyfin", &origin).unwrap();
    let configured = EgressScope::ConfiguredLocalService("jellyfin".into());
    assert!(
        policy
            .validate(
                &Url::parse("http://127.0.0.1:8096/Items").unwrap(),
                &configured
            )
            .await
            .is_ok()
    );
    assert_eq!(
        policy
            .validate(
                &Url::parse("http://169.254.169.254/latest/meta-data").unwrap(),
                &configured,
            )
            .await,
        Err(EgressPolicyError::LocalServiceOriginMismatch)
    );
    assert_eq!(
        policy
            .validate(
                &Url::parse("http://127.0.0.1:8096/Items").unwrap(),
                &EgressScope::ConfiguredLocalService("not-configured".into()),
            )
            .await,
        Err(EgressPolicyError::LocalServiceNotConfigured)
    );
}

#[test]
fn site_capability_isolated_by_site_host_and_expiry() {
    let capability = SiteAccessCapability::issue(
        "site-a",
        Some("account-a".into()),
        ["media.example.test".into()],
        "cap-a",
        Duration::from_secs(60),
    );
    assert!(
        capability
            .authorize(
                "site-a",
                &Url::parse("https://media.example.test/a").unwrap()
            )
            .is_ok()
    );
    assert_eq!(
        capability.authorize(
            "site-b",
            &Url::parse("https://media.example.test/a").unwrap()
        ),
        Err(SiteAccessError::SiteMismatch)
    );
    assert_eq!(
        capability.authorize(
            "site-a",
            &Url::parse("https://other.example.test/a").unwrap()
        ),
        Err(SiteAccessError::HostNotAllowed)
    );
    let expired = SiteAccessCapability::issue(
        "site-a",
        None,
        ["media.example.test".into()],
        "cap-expired",
        Duration::ZERO,
    );
    assert_eq!(
        expired.authorize(
            "site-a",
            &Url::parse("https://media.example.test/a").unwrap()
        ),
        Err(SiteAccessError::Expired)
    );
}

#[test]
fn secret_diagnostics_and_signed_urls_are_redacted() {
    let secret = "r008-fixture-secret";
    let diagnostic = format!("request failed Authorization: Bearer {secret}");
    assert!(!redact_text(&diagnostic, &[secret]).contains(secret));
    let url = Url::parse("https://cdn.example.test/video.m3u8?token=r008-fixture-secret").unwrap();
    assert_eq!(redact_url(&url), "https://cdn.example.test/video.m3u8");
}

#[test]
fn target_workflows_use_trusted_dispatch_and_bounded_least_privilege() {
    let workflow_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows");
    for entry in fs::read_dir(workflow_dir).unwrap() {
        let path = entry.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        if !text.contains("self-hosted") {
            continue;
        }
        assert!(!text.contains("pull_request:"), "{}", path.display());
        assert!(!text.contains("pull_request_target"), "{}", path.display());
        assert!(text.contains("workflow_dispatch:"), "{}", path.display());
        assert!(
            text.contains("permissions:\n  contents: read"),
            "{}",
            path.display()
        );
        assert!(text.contains("candidate_sha"), "{}", path.display());
        assert!(text.contains("timeout-minutes:"), "{}", path.display());
        assert!(text.contains("concurrency:"), "{}", path.display());
        assert!(!text.contains("secrets."), "{}", path.display());
    }
}

#[test]
fn runtime_sources_do_not_construct_shell_commands_from_untrusted_input() {
    let source_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in walk_rs(&source_dir) {
        let text = fs::read_to_string(&entry).unwrap();
        for forbidden in [
            "Command::new(\"sh\")",
            "Command::new(\"bash\")",
            "sh -c",
            "bash -c",
            ".arg(\"-c\")",
            "shell = true",
        ] {
            assert!(
                !text.contains(forbidden),
                "{} contains {forbidden}",
                entry.display()
            );
        }
    }
}

fn walk_rs(path: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(walk_rs(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files
}
