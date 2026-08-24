use axum::http::{HeaderMap, HeaderValue};
use gateway_core::{Binding, EgressScope, GatewayService, ProofPaths, UpstreamResource};
use generic_direct::GenericDirectAdapter;
use site_adapter_api::{SiteAdapterRegistry, StreamProtocol};
use std::env;
use std::net::IpAddr;
#[cfg(test)]
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use url::Url;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8787;
const DEFAULT_MP4: &str =
    "https://raw.githubusercontent.com/mediaelement/mediaelement-files/master/big_buck_bunny.mp4";
const DEFAULT_HLS: &str = "https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_ts/master.m3u8";

fn parse_bind_addr(value: Option<&str>) -> Result<IpAddr, String> {
    let value = value.unwrap_or(DEFAULT_BIND_ADDR);
    value
        .parse::<IpAddr>()
        .map_err(|_| format!("R001_BIND_ADDR must be an IP address, got {value:?}"))
}

fn configured_bind_addr() -> IpAddr {
    let configured = env::var("R001_BIND_ADDR").ok();
    parse_bind_addr(configured.as_deref()).unwrap_or_else(|error| panic!("{error}"))
}

#[tokio::main]
async fn main() {
    let port: u16 = env::var("R001_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let bind_addr = configured_bind_addr();
    let listener = TcpListener::bind((bind_addr, port))
        .await
        .expect("bind R001 server");
    let addr = listener.local_addr().expect("listener addr");

    let mut registry = SiteAdapterRegistry::default();
    registry
        .register(Arc::new(GenericDirectAdapter))
        .expect("register generic-direct");

    let service = GatewayService::new(1024);
    let ttl = Duration::from_secs(30 * 60);

    let mp4_input = env::var("R001_PUBLIC_MP4").unwrap_or_else(|_| DEFAULT_MP4.into());
    let mp4_locator = registry
        .recognize(&mp4_input)
        .expect("recognize public MP4");
    let mp4_media = registry.resolve(&mp4_locator).expect("resolve public MP4");
    let mp4_resource =
        GatewayService::resource_from_resolved(&mp4_media.streams[0], EgressScope::PublicWeb)
            .expect("public MP4 resource");
    let mp4_path = service.issue_path(
        Binding::new("r001-session", "public-mp4", 1, "primary"),
        mp4_resource,
        ttl,
    );

    let hls_input = env::var("R001_PUBLIC_HLS").unwrap_or_else(|_| DEFAULT_HLS.into());
    let hls_locator = registry
        .recognize(&hls_input)
        .expect("recognize public HLS");
    let hls_media = registry.resolve(&hls_locator).expect("resolve public HLS");
    let hls_resource =
        GatewayService::resource_from_resolved(&hls_media.streams[0], EgressScope::PublicWeb)
            .expect("public HLS resource");
    let hls_path = service.issue_path(
        Binding::new("r001-session", "public-hls", 1, "master"),
        hls_resource,
        ttl,
    );

    let fixture_path = env::var_os("R001_FIXTURE_MP4").map(PathBuf::from);
    service.configure_fixture_mp4(fixture_path.clone());
    let secret_path = fixture_path.map(|_| {
        service
            .configure_local_service(
                "r001-fixture",
                Url::parse(&format!("http://{addr}")).unwrap(),
            )
            .expect("configure R001 fixture local service");
        let mut secret_headers = HeaderMap::new();
        secret_headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer r001-fixture-secret"),
        );
        service.issue_path(
            Binding::new("r001-session", "protected-fixture", 1, "fixture"),
            UpstreamResource {
                url: Url::parse(&format!("http://{addr}/fixture/protected.mp4")).unwrap(),
                protocol: StreamProtocol::HttpFile,
                public_headers: HeaderMap::new(),
                secret_headers,
                egress_scope: EgressScope::ConfiguredLocalService("r001-fixture".into()),
            },
            ttl,
        )
    });

    let display_path = if env::var_os("R002_USE_FIXTURE_DISPLAY").is_some() {
        secret_path.clone()
    } else {
        Some(mp4_path.clone())
    };
    service.configure_proof_paths(ProofPaths {
        mp4_path: Some(mp4_path),
        display_path,
        hls_path: Some(hls_path),
        secret_path,
        chain: format!(
            "SiteAdapterRegistry -> {} -> SourceLocator(v{}) -> ResolvedMedia -> MediaGateway -> WebDisplay",
            mp4_locator.plugin_id, mp4_locator.locator_version
        ),
    });

    println!("R001 server listening on http://{addr}");
    println!(
        "R001 source chain uses plugin={} site={}",
        mp4_locator.plugin_id, mp4_locator.site_id
    );
    axum::serve(listener, service.router())
        .await
        .expect("serve R001");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_address_defaults_to_loopback() {
        assert_eq!(
            parse_bind_addr(None).expect("default bind address"),
            parse_bind_addr(Some(DEFAULT_BIND_ADDR)).unwrap()
        );
    }

    #[test]
    fn bind_address_accepts_explicit_ip() {
        assert_eq!(
            parse_bind_addr(Some("192.168.1.42")).expect("explicit bind address"),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 42))
        );
    }

    #[test]
    fn bind_address_rejects_hostnames_and_socket_strings() {
        for value in ["gateway.local", "0.0.0.0:8787", "not-an-ip"] {
            let error = parse_bind_addr(Some(value)).expect_err("invalid bind address accepted");
            assert!(error.contains("R001_BIND_ADDR must be an IP address"));
        }
    }

    #[tokio::test]
    async fn explicit_bind_address_is_used_by_listener() {
        let bind_addr = parse_bind_addr(Some("127.0.0.1")).expect("explicit bind address");
        let listener = TcpListener::bind((bind_addr, 0))
            .await
            .expect("bind explicit listener");
        assert_eq!(
            listener.local_addr().expect("listener address").ip(),
            bind_addr
        );
    }
}
