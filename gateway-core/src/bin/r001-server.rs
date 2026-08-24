use axum::http::{HeaderMap, HeaderValue};
use gateway_core::{Binding, EgressScope, GatewayService, ProofPaths, UpstreamResource};
use generic_direct::GenericDirectAdapter;
use site_adapter_api::{SiteAdapterRegistry, StreamProtocol};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use url::Url;

const DEFAULT_MP4: &str =
    "https://raw.githubusercontent.com/mediaelement/mediaelement-files/master/big_buck_bunny.mp4";
const DEFAULT_HLS: &str = "https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_ts/master.m3u8";

#[tokio::main]
async fn main() {
    let port: u16 = env::var("R001_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8787);
    let listener = TcpListener::bind(("127.0.0.1", port))
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
                egress_scope: EgressScope::FixtureLoopback,
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
