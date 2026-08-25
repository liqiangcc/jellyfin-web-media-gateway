use axum::Router;
use axum::http::header::{AUTHORIZATION, COOKIE, LOCATION, SET_COOKIE};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use gateway_core::{
    AccountState, AuthBoundaryError, CandidateValidation, CleanupResult, EgressPolicy, EgressScope,
    PendingIntent, PendingPlaybackAction, ScopedSiteHttpClient, SessionVault, SiteAccessCapability,
    SiteAccessContext, SiteAccessError, VaultError,
};
use site_adapter_api::SourceLocator;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

fn setup_account() -> (SessionVault, gateway_core::SiteSessionRef) {
    let vault = SessionVault::isolated_test();
    vault
        .register_account("site-a", "account-a", "Fixture account")
        .unwrap();
    let candidate = vault
        .create_fixture_candidate_session("site-a", "account-a", "old")
        .unwrap();
    let result = vault
        .validate_and_swap(&candidate, CandidateValidation::Valid)
        .unwrap();
    assert_eq!(result.cleanup, CleanupResult::NoPreviousSession);
    (vault, result.active_session)
}

#[test]
fn vault_models_account_state_and_failed_or_cancelled_rotation() {
    let (vault, old_active) = setup_account();
    assert_eq!(
        vault.account("site-a", "account-a").unwrap().state(),
        AccountState::Valid
    );

    let failed = vault
        .create_fixture_candidate_session("site-a", "account-a", "failed")
        .unwrap();
    assert_eq!(
        vault.validate_and_swap(&failed, CandidateValidation::Invalid),
        Err(VaultError::CandidateRejected)
    );
    assert_eq!(
        vault.active_session("site-a", "account-a").unwrap(),
        Some(old_active.clone())
    );

    let cancelled = vault
        .create_fixture_candidate_session("site-a", "account-a", "cancelled")
        .unwrap();
    vault.cancel_candidate(&cancelled).unwrap();
    assert_eq!(
        vault.active_session("site-a", "account-a").unwrap(),
        Some(old_active.clone())
    );

    let replacement = vault
        .create_fixture_candidate_session("site-a", "account-a", "new")
        .unwrap();
    let swapped = vault
        .validate_and_swap(&replacement, CandidateValidation::Valid)
        .unwrap();
    assert_eq!(swapped.replaced_session, Some(old_active.clone()));
    assert_eq!(swapped.cleanup, CleanupResult::PreviousSessionRemoved);
    assert!(!vault.has_session(&old_active));
    assert_eq!(
        vault.account("site-a", "account-a").unwrap().state(),
        AccountState::Valid
    );

    vault.mark_expired("site-a", "account-a").unwrap();
    assert_eq!(
        vault.account("site-a", "account-a").unwrap().state(),
        AccountState::Expired
    );
    vault.logout("site-a", "account-a").unwrap();
    let logged_out = vault.account("site-a", "account-a").unwrap();
    assert_eq!(logged_out.state(), AccountState::LoginRequired);
    assert_eq!(logged_out.active_session(), None);
}

#[test]
fn vault_keeps_one_active_account_per_site() {
    let (vault, first_active) = setup_account();
    vault
        .register_account("site-a", "account-b", "Second fixture account")
        .unwrap();
    let second_candidate = vault
        .create_fixture_candidate_session("site-a", "account-b", "second")
        .unwrap();
    let swapped = vault
        .validate_and_swap(&second_candidate, CandidateValidation::Valid)
        .unwrap();
    assert_eq!(swapped.replaced_session, Some(first_active.clone()));
    assert_eq!(vault.active_session("site-a", "account-a").unwrap(), None);
    assert_eq!(
        vault.account("site-a", "account-a").unwrap().state(),
        AccountState::LoginRequired
    );
    assert_eq!(
        vault
            .account("site-a", "account-b")
            .unwrap()
            .active_session(),
        Some(&swapped.active_session)
    );
    assert!(!vault.has_session(&first_active));
}

#[test]
fn capability_is_bound_to_site_account_host_expiry_and_active_session() {
    let (vault, active) = setup_account();
    let capability = SiteAccessCapability::issue_for_session(
        "site-a",
        "account-a",
        vec!["media.example.test".into()],
        "cap-old",
        active.session_id(),
        Duration::from_secs(60),
    );
    let media = Url::parse("https://media.example.test/a").unwrap();
    assert_eq!(
        capability.authorize("site-a", &media),
        Err(SiteAccessError::AccountMismatch)
    );
    assert!(
        vault
            .authorize_capability(&capability, "site-a", Some("account-a"), &media)
            .is_ok()
    );
    assert_eq!(
        vault.authorize_capability(&capability, "site-b", Some("account-a"), &media),
        Err(AuthBoundaryError::Capability(SiteAccessError::SiteMismatch))
    );
    assert_eq!(
        vault.authorize_capability(&capability, "site-a", Some("account-b"), &media),
        Err(AuthBoundaryError::Capability(
            SiteAccessError::AccountMismatch
        ))
    );
    assert_eq!(
        vault.authorize_capability(
            &capability,
            "site-a",
            Some("account-a"),
            &Url::parse("https://other.example.test/a").unwrap()
        ),
        Err(AuthBoundaryError::Capability(
            SiteAccessError::HostNotAllowed
        ))
    );

    let expired = SiteAccessCapability::issue_for_session(
        "site-a",
        "account-a",
        vec!["media.example.test".into()],
        "cap-expired",
        active.session_id(),
        Duration::ZERO,
    );
    assert_eq!(
        vault.authorize_capability(&expired, "site-a", Some("account-a"), &media),
        Err(AuthBoundaryError::Capability(SiteAccessError::Expired))
    );

    let replacement = vault
        .create_fixture_candidate_session("site-a", "account-a", "rotated")
        .unwrap();
    vault
        .validate_and_swap(&replacement, CandidateValidation::Valid)
        .unwrap();
    assert_eq!(
        vault.authorize_capability(&capability, "site-a", Some("account-a"), &media),
        Err(AuthBoundaryError::Capability(SiteAccessError::StaleSession))
    );
}

#[test]
fn pending_intent_is_recoverable_metadata_without_secret_fields() {
    let locator = SourceLocator {
        site_id: "site-a".into(),
        plugin_id: "fixture-public".into(),
        locator_version: 1,
        opaque_payload: "public-locator-42".into(),
    };
    let intent = PendingIntent::new(
        "intent-1",
        &locator,
        Some("display-1".into()),
        PendingPlaybackAction::Play,
        Some(7),
    );
    let encoded = serde_json::to_string(&intent).unwrap();
    assert!(encoded.contains("public-locator-42"));
    for forbidden in ["cookie", "authorization", "password", "profile", "token"] {
        assert!(
            !encoded.to_ascii_lowercase().contains(forbidden),
            "{forbidden}"
        );
    }
    assert_eq!(intent.to_source_locator(), locator);
}

#[test]
fn vault_and_capability_diagnostics_redact_secret_sentinels() {
    let capability = SiteAccessCapability::issue(
        "site-a",
        Some("account-a".into()),
        vec!["media.example.test".into()],
        "capability-ref",
        Duration::from_secs(60),
    );
    let capability_debug = format!("{capability:?}");
    assert!(!capability_debug.contains("sentinel"));
    assert!(!capability_debug.contains("cookie"));
    assert!(!capability_debug.contains("authorization"));
}

#[tokio::test]
async fn controlled_http_injects_only_vault_auth_and_rechecks_redirects() {
    async fn auth_endpoint(headers: HeaderMap) -> (StatusCode, HeaderMap, &'static str) {
        let mut response_headers = HeaderMap::new();
        response_headers.insert(SET_COOKIE, "response-secret".parse().unwrap());
        response_headers.insert(AUTHORIZATION, "Bearer response-secret".parse().unwrap());
        if headers.get(COOKIE).and_then(|value| value.to_str().ok()) == Some("session-cookie-old")
            && headers
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                == Some("Bearer session-token-old")
        {
            (StatusCode::OK, response_headers, "authenticated")
        } else {
            (StatusCode::UNAUTHORIZED, response_headers, "missing-auth")
        }
    }

    async fn redirect_endpoint() -> (StatusCode, HeaderMap) {
        let mut headers = HeaderMap::new();
        headers.insert(LOCATION, "/auth".parse().unwrap());
        (StatusCode::FOUND, headers)
    }

    async fn bad_redirect_endpoint() -> (StatusCode, HeaderMap) {
        let mut headers = HeaderMap::new();
        headers.insert(LOCATION, "http://127.0.0.1:1/auth".parse().unwrap());
        (StatusCode::FOUND, headers)
    }

    let app = Router::new()
        .route("/auth", get(auth_endpoint))
        .route("/redirect", get(redirect_endpoint))
        .route("/bad-redirect", get(bad_redirect_endpoint));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let (vault, active) = setup_account();
    let mut policy = EgressPolicy::default();
    policy
        .configure_local_service(
            "fixture",
            &Url::parse(&format!("http://127.0.0.1:{}/", address.port())).unwrap(),
        )
        .unwrap();
    let egress = Arc::new(policy);
    let client = ScopedSiteHttpClient::new(
        vault.clone(),
        egress.clone(),
        EgressScope::ConfiguredLocalService("fixture".into()),
        SiteAccessContext::issue("site-a", Some("account-a".into())),
    );
    let capability = SiteAccessCapability::issue_for_session(
        "site-a",
        "account-a",
        vec!["127.0.0.1".into()],
        "cap-http",
        active.session_id(),
        Duration::from_secs(60),
    );
    let omitted_account_client = ScopedSiteHttpClient::new(
        vault.clone(),
        egress.clone(),
        EgressScope::ConfiguredLocalService("fixture".into()),
        SiteAccessContext::issue("site-a", None),
    );
    let wrong_account_client = ScopedSiteHttpClient::new(
        vault.clone(),
        egress.clone(),
        EgressScope::ConfiguredLocalService("fixture".into()),
        SiteAccessContext::issue("site-a", Some("account-b".into())),
    );
    let wrong_site_client = ScopedSiteHttpClient::new(
        vault,
        egress,
        EgressScope::ConfiguredLocalService("fixture".into()),
        SiteAccessContext::issue("site-b", Some("account-a".into())),
    );
    let auth_url = Url::parse(&format!("http://127.0.0.1:{}/auth", address.port())).unwrap();
    assert_eq!(
        omitted_account_client
            .request(&capability, reqwest::Method::GET, auth_url.clone())
            .await,
        Err(AuthBoundaryError::Capability(
            SiteAccessError::AccountMismatch
        ))
    );
    assert_eq!(
        wrong_account_client
            .request(&capability, reqwest::Method::GET, auth_url.clone())
            .await,
        Err(AuthBoundaryError::Capability(
            SiteAccessError::AccountMismatch
        ))
    );
    assert_eq!(
        wrong_site_client
            .request(&capability, reqwest::Method::GET, auth_url)
            .await,
        Err(AuthBoundaryError::Capability(SiteAccessError::SiteMismatch))
    );
    let response = client
        .request(
            &capability,
            reqwest::Method::GET,
            Url::parse(&format!("http://127.0.0.1:{}/redirect", address.port())).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body, "authenticated");
    assert!(!response.headers.contains_key(SET_COOKIE));
    assert!(!response.headers.contains_key(AUTHORIZATION));

    let invalid_redirect = client
        .request(
            &capability,
            reqwest::Method::GET,
            Url::parse(&format!("http://127.0.0.1:{}/bad-redirect", address.port())).unwrap(),
        )
        .await;
    assert_eq!(
        invalid_redirect,
        Err(AuthBoundaryError::Egress(
            gateway_core::EgressPolicyError::LocalServiceOriginMismatch
        ))
    );
}
