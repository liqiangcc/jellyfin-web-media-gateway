//! Production-oriented Gateway composition and bounded deployment config.
//!
//! This crate is the executable composition root. `gateway-core` remains
//! site-agnostic; concrete adapters are registered here and the only account
//! registration is the server-owned `GatewayService::configure_auth_account`
//! seam.

use bilibili::BilibiliAdapter;
use gateway_core::{GatewayService, HttpAuthorityError, VaultError};
use generic_direct::GenericDirectAdapter;
use generic_ytdlp::register_prep_adapter;
use site_adapter_api::{AdapterError, SiteAdapterRegistry};
use std::env;
use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;
use url::Url;

pub const BILIBILI_SITE_ID: &str = bilibili::SITE_ID;
pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8787;
pub const DEFAULT_MAX_CAPABILITIES: usize = 1024;
pub const BIND_ADDR_ENV: &str = "GATEWAY_BIND_ADDR";
pub const PORT_ENV: &str = "GATEWAY_PORT";
pub const HTTP_AUTHORITY_ENV: &str = "GATEWAY_HTTP_AUTHORITY";
pub const BILIBILI_ACCOUNT_REF_ENV: &str = "GATEWAY_BILIBILI_ACCOUNT_REF";
pub const BILIBILI_ACCOUNT_LABEL_ENV: &str = "GATEWAY_BILIBILI_ACCOUNT_LABEL";

const MAX_ACCOUNT_REF: usize = 128;
const MAX_ACCOUNT_LABEL: usize = 128;
const MAX_AUTHORITY: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayRuntimeConfig {
    pub bind_addr: IpAddr,
    pub port: u16,
    pub http_authority: Url,
    pub bilibili_account_ref: String,
    pub bilibili_account_label: String,
    pub max_capabilities: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeConfigError {
    Missing(&'static str),
    Invalid(&'static str),
    InvalidValue(&'static str),
    Adapter(AdapterError),
    Authority(HttpAuthorityError),
    Vault(VaultError),
}

impl fmt::Display for RuntimeConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(name) => write!(formatter, "missing required configuration {name}"),
            Self::Invalid(name) => write!(formatter, "invalid configuration {name}"),
            Self::InvalidValue(name) => write!(formatter, "invalid value for {name}"),
            Self::Adapter(error) => write!(formatter, "adapter registration failed: {error:?}"),
            Self::Authority(error) => write!(formatter, "HTTP authority rejected: {error:?}"),
            Self::Vault(error) => write!(formatter, "account registration failed: {error:?}"),
        }
    }
}

impl std::error::Error for RuntimeConfigError {}

impl From<AdapterError> for RuntimeConfigError {
    fn from(error: AdapterError) -> Self {
        Self::Adapter(error)
    }
}

impl GatewayRuntimeConfig {
    pub fn from_env() -> Result<Self, RuntimeConfigError> {
        Self::from_values(
            env::var(BIND_ADDR_ENV).ok().as_deref(),
            env::var(PORT_ENV).ok().as_deref(),
            env::var(HTTP_AUTHORITY_ENV).ok().as_deref(),
            env::var(BILIBILI_ACCOUNT_REF_ENV).ok().as_deref(),
            env::var(BILIBILI_ACCOUNT_LABEL_ENV).ok().as_deref(),
        )
    }

    pub fn from_values(
        bind_addr: Option<&str>,
        port: Option<&str>,
        http_authority: Option<&str>,
        account_ref: Option<&str>,
        account_label: Option<&str>,
    ) -> Result<Self, RuntimeConfigError> {
        let bind_addr = bind_addr
            .unwrap_or(DEFAULT_BIND_ADDR)
            .parse::<IpAddr>()
            .map_err(|_| RuntimeConfigError::Invalid(BIND_ADDR_ENV))?;
        if bind_addr.is_unspecified() || bind_addr.is_multicast() {
            return Err(RuntimeConfigError::InvalidValue(BIND_ADDR_ENV));
        }

        let port = port
            .map(|value| value.parse::<u16>())
            .transpose()
            .map_err(|_| RuntimeConfigError::Invalid(PORT_ENV))?
            .unwrap_or(DEFAULT_PORT);
        if port == 0 {
            return Err(RuntimeConfigError::InvalidValue(PORT_ENV));
        }

        let authority = http_authority.ok_or(RuntimeConfigError::Missing(HTTP_AUTHORITY_ENV))?;
        if authority.len() > MAX_AUTHORITY {
            return Err(RuntimeConfigError::Invalid(HTTP_AUTHORITY_ENV));
        }
        let http_authority =
            Url::parse(authority).map_err(|_| RuntimeConfigError::Invalid(HTTP_AUTHORITY_ENV))?;
        validate_authority(&http_authority)?;
        if http_authority.port_or_known_default() != Some(port) {
            return Err(RuntimeConfigError::InvalidValue(HTTP_AUTHORITY_ENV));
        }

        let bilibili_account_ref = bounded_ref(
            account_ref.ok_or(RuntimeConfigError::Missing(BILIBILI_ACCOUNT_REF_ENV))?,
            MAX_ACCOUNT_REF,
            BILIBILI_ACCOUNT_REF_ENV,
        )?;
        let bilibili_account_label = bounded_label(
            account_label.ok_or(RuntimeConfigError::Missing(BILIBILI_ACCOUNT_LABEL_ENV))?,
            MAX_ACCOUNT_LABEL,
            BILIBILI_ACCOUNT_LABEL_ENV,
        )?;

        Ok(Self {
            bind_addr,
            port,
            http_authority,
            bilibili_account_ref,
            bilibili_account_label,
            max_capabilities: DEFAULT_MAX_CAPABILITIES,
        })
    }
}

fn validate_authority(authority: &Url) -> Result<(), RuntimeConfigError> {
    if !matches!(authority.scheme(), "http" | "https")
        || !authority.username().is_empty()
        || authority.password().is_some()
        || !matches!(authority.path(), "" | "/")
        || authority.query().is_some()
        || authority.fragment().is_some()
        || authority.host_str().is_none()
        || authority.port_or_known_default().is_none()
    {
        return Err(RuntimeConfigError::Invalid(HTTP_AUTHORITY_ENV));
    }
    Ok(())
}

fn bounded_ref(value: &str, max: usize, name: &'static str) -> Result<String, RuntimeConfigError> {
    let lower = value.to_ascii_lowercase();
    if value.is_empty()
        || value.len() > max
        || value.chars().any(char::is_control)
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
        || ["cookie", "authorization", "bearer", "token", "secret"]
            .iter()
            .any(|marker| lower.contains(marker))
    {
        return Err(RuntimeConfigError::Invalid(name));
    }
    Ok(value.to_owned())
}

fn bounded_label(
    value: &str,
    max: usize,
    name: &'static str,
) -> Result<String, RuntimeConfigError> {
    let lower = value.to_ascii_lowercase();
    if value.is_empty()
        || value.len() > max
        || value.chars().any(char::is_control)
        || [
            "cookie",
            "authorization",
            "bearer",
            "password",
            "token",
            "secret",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(RuntimeConfigError::Invalid(name));
    }
    Ok(value.to_owned())
}

pub fn production_registry() -> Result<Arc<SiteAdapterRegistry>, RuntimeConfigError> {
    let mut registry = SiteAdapterRegistry::default();
    BilibiliAdapter::register(&mut registry)?;
    registry.register(Arc::new(GenericDirectAdapter))?;
    register_prep_adapter(&mut registry)?;
    Ok(Arc::new(registry))
}

pub fn build_service(config: &GatewayRuntimeConfig) -> Result<GatewayService, RuntimeConfigError> {
    let service = GatewayService::with_registry(config.max_capabilities, production_registry()?);
    service
        .configure_http_authority(config.http_authority.clone())
        .map_err(RuntimeConfigError::Authority)?;
    service
        .configure_auth_account(
            BILIBILI_SITE_ID,
            config.bilibili_account_ref.clone(),
            config.bilibili_account_label.clone(),
        )
        .map_err(RuntimeConfigError::Vault)?;
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn config() -> GatewayRuntimeConfig {
        GatewayRuntimeConfig::from_values(
            None,
            None,
            Some("http://127.0.0.1:8787"),
            Some("home-bilibili"),
            Some("Home Bilibili"),
        )
        .unwrap()
    }

    #[test]
    fn production_registry_routes_bilibili_and_safe_generic_adapters() {
        let registry = production_registry().unwrap();
        let bilibili = registry
            .recognize("https://www.bilibili.com/video/BV1xx411c7mD/")
            .unwrap();
        assert_eq!(bilibili.site_id, BILIBILI_SITE_ID);
        assert_eq!(bilibili.plugin_id, bilibili::PLUGIN_ID);
        let direct = registry
            .recognize("https://media.example.invalid/video.mp4")
            .unwrap();
        assert_eq!(direct.plugin_id, "generic-direct");
        let fallback = registry
            .recognize("https://example.invalid/watch/1")
            .unwrap();
        assert_eq!(fallback.plugin_id, "generic-ytdlp");
    }

    #[test]
    fn config_is_private_by_default_and_rejects_secret_or_unbounded_values() {
        let config = config();
        assert_eq!(config.bind_addr, "127.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(config.port, DEFAULT_PORT);
        assert!(
            GatewayRuntimeConfig::from_values(
                Some("0.0.0.0"),
                None,
                Some("http://127.0.0.1:8787"),
                Some("home-bilibili"),
                Some("Home Bilibili"),
            )
            .is_err()
        );
        assert!(
            GatewayRuntimeConfig::from_values(
                None,
                None,
                Some("http://127.0.0.1:8787"),
                Some("cookie-value"),
                Some("Home Bilibili"),
            )
            .is_err()
        );
        assert!(
            GatewayRuntimeConfig::from_values(
                None,
                None,
                Some("http://user:password@127.0.0.1:8787"),
                Some("home-bilibili"),
                Some("Home Bilibili"),
            )
            .is_err()
        );
        assert!(
            GatewayRuntimeConfig::from_values(
                None,
                None,
                None,
                Some("home-bilibili"),
                Some("Home Bilibili"),
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn fake_runtime_reaches_auth_start_for_registered_account_and_rejects_unregistered() {
        let config = config();
        let registry = production_registry().unwrap();
        let service = GatewayService::with_fake_auth_routes(config.max_capabilities, registry);
        service
            .configure_http_authority(config.http_authority)
            .unwrap();
        service
            .configure_auth_account(
                BILIBILI_SITE_ID,
                config.bilibili_account_ref,
                config.bilibili_account_label,
            )
            .unwrap();
        let app = service.router();

        let registered = app.clone().oneshot(
            Request::post("/api/v1/auth/attempts")
                .header("host", "127.0.0.1:8787")
                .header("origin", "http://127.0.0.1:8787")
                .header("x-forwarded-proto", "http")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"request_id":"req-1","site_id":"bilibili","account_ref":"home-bilibili"}"#))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(registered.status(), StatusCode::OK);
        let registered_body = to_bytes(registered.into_body(), 4096).await.unwrap();
        let registered: serde_json::Value = serde_json::from_slice(&registered_body).unwrap();
        let attempt_id = registered["attempt_id"].as_str().unwrap();
        let view_capability = registered["view_capability"].as_str().unwrap();
        let panel_capability = registered["panel_capability"].as_str().unwrap();
        assert!(view_capability.starts_with("view-"));
        assert!(panel_capability.starts_with("panel-"));

        let missing_capability = app
            .clone()
            .oneshot(
                Request::get(format!("/api/v1/auth/attempts/{attempt_id}/events?after=0"))
                    .header("host", "127.0.0.1:8787")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing_capability.status(), StatusCode::FORBIDDEN);
        let authorized_events = app
            .clone()
            .oneshot(
                Request::get(format!("/api/v1/auth/attempts/{attempt_id}/events?after=0"))
                    .header("host", "127.0.0.1:8787")
                    .header("x-gateway-auth-view-capability", view_capability)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(authorized_events.status(), StatusCode::OK);

        let unregistered = app.oneshot(
            Request::post("/api/v1/auth/attempts")
                .header("host", "127.0.0.1:8787")
                .header("origin", "http://127.0.0.1:8787")
                .header("x-forwarded-proto", "http")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"request_id":"req-2","site_id":"bilibili","account_ref":"other-account"}"#))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(unregistered.status(), StatusCode::CONFLICT);
        let body = to_bytes(unregistered.into_body(), 1024).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["code"], "AUTH_ACCOUNT_NOT_FOUND");
    }
}
