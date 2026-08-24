use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::header::{
    ACCEPT_RANGES, AUTHORIZATION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, LOCATION, RANGE,
};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use gateway_core::{Binding, EgressScope, GatewayService, UpstreamResource};
use site_adapter_api::StreamProtocol;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::{sleep, timeout};
use url::Url;

#[derive(Default)]
struct FixtureStats {
    hits: AtomicUsize,
    protected_hits: AtomicUsize,
}

async fn spawn_fixture() -> (String, Arc<FixtureStats>) {
    let stats = Arc::new(FixtureStats::default());
    let app = Router::new()
        .route("/range.mp4", get(range_file))
        .route("/no-range.mp4", get(no_range_file))
        .route("/protected.mp4", get(protected_file))
        .route("/missing.mp4", get(|| async { StatusCode::NOT_FOUND }))
        .route("/forbidden.mp4", get(|| async { StatusCode::FORBIDDEN }))
        .route(
            "/redirect.mp4",
            get(|| async { (StatusCode::FOUND, [(LOCATION, "/range.mp4")]) }),
        )
        .route(
            "/redirect-private.mp4",
            get(|| async {
                (
                    StatusCode::FOUND,
                    [(LOCATION, "http://169.254.169.254/latest/meta-data")],
                )
            }),
        )
        .route("/slow.mp4", get(slow_file))
        .route("/hls/master.m3u8", get(hls_master))
        .route("/hls/variant.m3u8", get(hls_variant))
        .route(
            "/hls/seg0.ts",
            get(|| async { ([(CONTENT_TYPE, "video/mp2t")], vec![7u8; 4096]) }),
        )
        .route("/hls/missing.ts", get(|| async { StatusCode::NOT_FOUND }))
        .route("/hls/interrupted.ts", get(interrupted_segment))
        .with_state(stats.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), stats)
}

async fn spawn_gateway(service: GatewayService) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, service.router()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn range_file(State(stats): State<Arc<FixtureStats>>, headers: HeaderMap) -> Response {
    stats.hits.fetch_add(1, Ordering::SeqCst);
    let data: Vec<u8> = (0..=255).cycle().take(8192).collect();
    if let Some(raw) = headers.get(RANGE).and_then(|value| value.to_str().ok()) {
        let value = raw.strip_prefix("bytes=").unwrap();
        let (start, end) = value.split_once('-').unwrap();
        let start: usize = start.parse().unwrap();
        let end: usize = if end.is_empty() {
            data.len() - 1
        } else {
            end.parse().unwrap()
        };
        let body = data[start..=end].to_vec();
        return Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(CONTENT_TYPE, "video/mp4")
            .header(ACCEPT_RANGES, "bytes")
            .header(CONTENT_RANGE, format!("bytes {start}-{end}/{}", data.len()))
            .header(CONTENT_LENGTH, body.len().to_string())
            .body(Body::from(body))
            .unwrap();
    }
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "video/mp4")
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, data.len().to_string())
        .body(Body::from(data))
        .unwrap()
}

async fn no_range_file(State(stats): State<Arc<FixtureStats>>) -> Response {
    stats.hits.fetch_add(1, Ordering::SeqCst);
    ([(CONTENT_TYPE, "video/mp4")], vec![1u8; 4096]).into_response()
}

async fn protected_file(State(stats): State<Arc<FixtureStats>>, headers: HeaderMap) -> Response {
    stats.hits.fetch_add(1, Ordering::SeqCst);
    let authorized = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        == Some("Bearer fixture-test-secret");
    if !authorized {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    stats.protected_hits.fetch_add(1, Ordering::SeqCst);
    ([(CONTENT_TYPE, "video/mp4")], vec![3u8; 2048]).into_response()
}

async fn slow_file(State(stats): State<Arc<FixtureStats>>) -> Response {
    stats.hits.fetch_add(1, Ordering::SeqCst);
    let body = stream::unfold(0usize, |index| async move {
        if index >= 200 {
            None
        } else {
            sleep(Duration::from_millis(2)).await;
            Some((
                Ok::<Bytes, std::io::Error>(Bytes::from(vec![9u8; 1024])),
                index + 1,
            ))
        }
    });
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "video/mp4")
        .body(Body::from_stream(body))
        .unwrap()
}

async fn hls_master(State(stats): State<Arc<FixtureStats>>) -> Response {
    stats.hits.fetch_add(1, Ordering::SeqCst);
    let text = "#EXTM3U\n#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"a\",URI=\"variant.m3u8?audio=1\"\n#EXT-X-STREAM-INF:BANDWIDTH=100000\nvariant.m3u8?quality=1\n";
    ([(CONTENT_TYPE, "application/vnd.apple.mpegurl")], text).into_response()
}

async fn hls_variant(State(stats): State<Arc<FixtureStats>>) -> Response {
    stats.hits.fetch_add(1, Ordering::SeqCst);
    let text = "#EXTM3U\n#EXTINF:2,\nseg0.ts?part=0\n#EXTINF:2,\nmissing.ts\n#EXTINF:2,\ninterrupted.ts\n#EXT-X-ENDLIST\n";
    ([(CONTENT_TYPE, "application/vnd.apple.mpegurl")], text).into_response()
}

async fn interrupted_segment(State(stats): State<Arc<FixtureStats>>) -> Response {
    stats.hits.fetch_add(1, Ordering::SeqCst);
    let body = stream::iter(vec![
        Ok::<Bytes, std::io::Error>(Bytes::from_static(b"partial")),
        Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "fixture interrupted",
        )),
    ]);
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "video/mp2t")
        .header(CONTENT_LENGTH, "128")
        .body(Body::from_stream(body))
        .unwrap()
}

fn resource(url: &str, protocol: StreamProtocol) -> UpstreamResource {
    UpstreamResource {
        url: Url::parse(url).unwrap(),
        protocol,
        public_headers: HeaderMap::new(),
        secret_headers: HeaderMap::new(),
        egress_scope: EgressScope::FixtureLoopback,
    }
}

async fn wait_active_zero(service: &GatewayService) {
    timeout(Duration::from_secs(2), async {
        while service.active_streams() != 0 {
            sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("active stream cleanup");
}

#[tokio::test]
async fn range_and_capability_binding_are_enforced_before_upstream() {
    let (fixture, stats) = spawn_fixture().await;
    let service = GatewayService::new(64);
    let binding = Binding::new("s1", "i1", 7, "video");
    let path = service.issue_path(
        binding,
        resource(&format!("{fixture}/range.mp4"), StreamProtocol::HttpFile),
        Duration::from_secs(30),
    );
    let base = spawn_gateway(service.clone()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{base}{path}"))
        .header(RANGE, "bytes=100-199")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        response.headers().get(CONTENT_RANGE).unwrap(),
        "bytes 100-199/8192"
    );
    assert_eq!(response.bytes().await.unwrap().len(), 100);
    wait_active_zero(&service).await;

    let hits = stats.hits.load(Ordering::SeqCst);
    for bad_path in [
        path.replacen("/s1/", "/other-session/", 1),
        path.replacen("/i1/", "/other-item/", 1),
        path.replacen("/7/", "/8/", 1),
        path.replacen("/video", "/other-resource", 1),
    ] {
        let response = client
            .get(format!("{base}{bad_path}"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    assert_eq!(stats.hits.load(Ordering::SeqCst), hits);

    assert_eq!(
        client
            .get(format!("{base}/stream/not-a-token/s1/i1/7/video"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        client
            .get(format!("{base}/stream?url=http://127.0.0.1/anything"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(stats.hits.load(Ordering::SeqCst), hits);
}

#[tokio::test]
async fn expired_secret_redirect_and_failure_boundaries_are_deterministic() {
    let (fixture, stats) = spawn_fixture().await;
    let service = GatewayService::new(64);
    let base = spawn_gateway(service.clone()).await;
    let client = reqwest::Client::new();

    let mut protected = resource(
        &format!("{fixture}/protected.mp4"),
        StreamProtocol::HttpFile,
    );
    protected.secret_headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer fixture-test-secret"),
    );
    let path = service.issue_path(
        Binding::new("s", "protected", 1, "video"),
        protected,
        Duration::from_secs(30),
    );
    assert_eq!(
        client
            .get(format!("{fixture}/protected.mp4"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let response = client.get(format!("{base}{path}")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(!path.contains("fixture-test-secret"));
    assert_eq!(response.bytes().await.unwrap().len(), 2048);
    assert_eq!(stats.protected_hits.load(Ordering::SeqCst), 1);

    for (name, expected) in [
        ("missing.mp4", StatusCode::NOT_FOUND),
        ("forbidden.mp4", StatusCode::FORBIDDEN),
    ] {
        let path = service.issue_path(
            Binding::new("s", name, 1, "video"),
            resource(&format!("{fixture}/{name}"), StreamProtocol::HttpFile),
            Duration::from_secs(30),
        );
        assert_eq!(
            client
                .get(format!("{base}{path}"))
                .send()
                .await
                .unwrap()
                .status(),
            expected
        );
    }

    let redirect = service.issue_path(
        Binding::new("s", "redirect", 1, "video"),
        resource(&format!("{fixture}/redirect.mp4"), StreamProtocol::HttpFile),
        Duration::from_secs(30),
    );
    assert_eq!(
        client
            .get(format!("{base}{redirect}"))
            .header(RANGE, "bytes=0-31")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::PARTIAL_CONTENT
    );

    let blocked = service.issue_path(
        Binding::new("s", "redirect-private", 1, "video"),
        resource(
            &format!("{fixture}/redirect-private.mp4"),
            StreamProtocol::HttpFile,
        ),
        Duration::from_secs(30),
    );
    assert_eq!(
        client
            .get(format!("{base}{blocked}"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::BAD_GATEWAY
    );

    let unsupported = service.issue_path(
        Binding::new("s", "no-range", 1, "video"),
        resource(&format!("{fixture}/no-range.mp4"), StreamProtocol::HttpFile),
        Duration::from_secs(30),
    );
    let response = client
        .get(format!("{base}{unsupported}"))
        .header(RANGE, "bytes=0-10")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(response.text().await.unwrap(), "UPSTREAM_RANGE_UNSUPPORTED");

    let expired = service.issue_path(
        Binding::new("s", "expired", 1, "video"),
        resource(&format!("{fixture}/range.mp4"), StreamProtocol::HttpFile),
        Duration::from_millis(5),
    );
    sleep(Duration::from_millis(20)).await;
    assert_eq!(
        client
            .get(format!("{base}{expired}"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::GONE
    );
}

#[tokio::test]
async fn hls_manifest_query_segment_and_interruption_have_concrete_results() {
    let (fixture, _stats) = spawn_fixture().await;
    let service = GatewayService::new(128);
    let base = spawn_gateway(service.clone()).await;
    let client = reqwest::Client::new();
    let path = service.issue_path(
        Binding::new("s", "hls", 1, "master"),
        resource(
            &format!("{fixture}/hls/master.m3u8?token=upstream-secret-query"),
            StreamProtocol::Hls,
        ),
        Duration::from_secs(30),
    );
    let master = client
        .get(format!("{base}{path}"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(master.contains("/stream/"));
    assert!(!master.contains(&fixture));
    assert!(!master.contains("upstream-secret-query"));
    assert!(!master.contains("quality=1"));
    assert!(!master.contains("audio=1"));

    let variant_path = master
        .lines()
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .unwrap();
    let variant = client
        .get(format!("{base}{variant_path}"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let children: Vec<_> = variant
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    assert_eq!(children.len(), 3);
    let segment = client
        .get(format!("{base}{}", children[0]))
        .send()
        .await
        .unwrap();
    assert_eq!(segment.status(), StatusCode::OK);
    assert_eq!(segment.bytes().await.unwrap().len(), 4096);
    assert_eq!(
        client
            .get(format!("{base}{}", children[1]))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    let interrupted = client
        .get(format!("{base}{}", children[2]))
        .send()
        .await
        .unwrap();
    let partial = interrupted.bytes().await.unwrap();
    assert_eq!(partial.as_ref(), b"partial");
    wait_active_zero(&service).await;
}

#[tokio::test]
async fn repeated_cleanup_is_bounded() {
    let (fixture, _stats) = spawn_fixture().await;
    let service = GatewayService::new(32);
    let base = spawn_gateway(service.clone()).await;
    let client = reqwest::Client::new();
    let path = service.issue_path(
        Binding::new("s", "slow", 1, "video"),
        resource(&format!("{fixture}/slow.mp4"), StreamProtocol::HttpFile),
        Duration::from_secs(60),
    );

    for _ in 0..100 {
        let response = client.get(format!("{base}{path}")).send().await.unwrap();
        let mut body = response.bytes_stream();
        assert!(body.next().await.unwrap().is_ok());
        drop(body);
        wait_active_zero(&service).await;
    }
    assert_eq!(service.active_streams(), 0);
    assert!(service.capability_count() <= service.max_capabilities());
    assert_eq!(service.capability_count(), 1);
}
