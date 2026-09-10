use axum::body::Body;
use axum::extract::State;
use axum::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use gateway_core::{
    DeliveryBinding, DeliveryInputCapability, DeliveryRequest, EgressScope,
    HttpFileDeliveryBroker, GatewayService, UpstreamResource, DEFAULT_DELIVERY_TIMEOUT,
    DEFAULT_DELIVERY_TTL,
    MAX_DELIVERY_OUTPUT_BYTES,
};
use site_adapter_api::{MediaShapeV1, MediaTrack, MediaTrackKind, StreamProtocol};
use std::env;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use url::Url;

#[derive(Clone)]
struct Fixture {
    video: Arc<Vec<u8>>,
    audio: Arc<Vec<u8>>,
}

async fn fixture_video(State(fixture): State<Fixture>) -> Response {
    fixture_response(fixture.video.as_slice(), "video/mp4")
}

async fn fixture_audio(State(fixture): State<Fixture>) -> Response {
    fixture_response(fixture.audio.as_slice(), "audio/mp4")
}

fn fixture_response(bytes: &[u8], content_type: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_LENGTH, bytes.len().to_string())
        .body(Body::from(bytes.to_vec()))
        .expect("fixture response")
}

fn read_fixture(name: &str) -> Vec<u8> {
    let path = env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{name} must point to a hosted synthetic fixture"));
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("read {name}: {error}"));
    assert!(!bytes.is_empty(), "{name} must not be empty");
    bytes
}

fn bind_addr() -> IpAddr {
    env::var("MEDIA_DELIVERY_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1".into())
        .parse()
        .expect("MEDIA_DELIVERY_BIND_ADDR must be an IP address")
}

fn port() -> u16 {
    env::var("MEDIA_DELIVERY_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8787)
}

#[tokio::main]
async fn main() {
    let fixture = Fixture {
        video: Arc::new(read_fixture("MEDIA_DELIVERY_FIXTURE_VIDEO")),
        audio: Arc::new(read_fixture("MEDIA_DELIVERY_FIXTURE_AUDIO")),
    };
    let fixture_listener = TcpListener::bind((bind_addr(), 0))
        .await
        .expect("bind synthetic fixture");
    let fixture_addr = fixture_listener.local_addr().expect("fixture address");
    let fixture_app = Router::new()
        .route("/video.mp4", get(fixture_video))
        .route("/audio.m4a", get(fixture_audio))
        .with_state(fixture);
    tokio::spawn(async move {
        axum::serve(fixture_listener, fixture_app)
            .await
            .expect("serve synthetic fixture");
    });

    let listener = TcpListener::bind((bind_addr(), port()))
        .await
        .expect("bind media delivery harness");
    let address = listener.local_addr().expect("Gateway address");
    let fixture_origin = Url::parse(&format!("http://{fixture_addr}")).unwrap();
    let service = GatewayService::new(64);
    service
        .configure_local_service("synthetic-fixture", fixture_origin.clone())
        .expect("configure synthetic fixture egress");
    service
        .configure_http_authority(Url::parse(&format!("http://{address}")).unwrap())
        .expect("configure Gateway HTTP authority");
    let session_id = service
        .seed_control_ui_harness_session()
        .expect("seed Gateway Playback authority");
    let binding = DeliveryBinding::new(&session_id, "control-ui-item", 1, 0, 1, "group-1");
    let resource = |path: &str| UpstreamResource {
        url: Url::parse(&format!("{fixture_origin}{path}")).unwrap(),
        protocol: StreamProtocol::HttpFile,
        public_headers: Default::default(),
        secret_headers: Default::default(),
        egress_scope: EgressScope::ConfiguredLocalService("synthetic-fixture".into()),
    };
    let video = DeliveryInputCapability::new_server_owned("video-1", "video-ref", resource("/video.mp4"))
        .expect("video capability");
    let audio = DeliveryInputCapability::new_server_owned("audio-1", "audio-ref", resource("/audio.m4a"))
        .expect("audio capability");
    let shape = MediaShapeV1 {
        version: site_adapter_api::MEDIA_SHAPE_VERSION,
        tracks: vec![
            MediaTrack {
                id: "video-1".into(),
                kind: MediaTrackKind::Video,
                group_id: Some("group-1".into()),
                protocol: StreamProtocol::HttpFile,
                codec: Some("avc1".into()),
                container: Some("mp4".into()),
                mime_type: Some("video/mp4".into()),
                width: Some(64),
                height: Some(64),
                bitrate: None,
                language: None,
                access_ref: Some("video-ref".into()),
                expires_at: Some(unix_seconds() + 300),
            },
            MediaTrack {
                id: "audio-1".into(),
                kind: MediaTrackKind::Audio,
                group_id: Some("group-1".into()),
                protocol: StreamProtocol::HttpFile,
                codec: Some("mp4a".into()),
                container: Some("m4a".into()),
                mime_type: Some("audio/mp4".into()),
                width: None,
                height: None,
                bitrate: None,
                language: Some("und".into()),
                access_ref: Some("audio-ref".into()),
                expires_at: Some(unix_seconds() + 300),
            },
        ],
    };
    let request = DeliveryRequest {
        binding: binding.clone(),
        shape,
        inputs: vec![video, audio],
        authority: service.playback_delivery_authority(session_id.clone()),
        timeout: DEFAULT_DELIVERY_TIMEOUT,
        output_ttl: DEFAULT_DELIVERY_TTL,
        max_output_bytes: MAX_DELIVERY_OUTPUT_BYTES,
    };
    let start_path = service
        .queue_media_delivery(request, Arc::new(HttpFileDeliveryBroker::new()))
        .expect("queue server-owned delivery");
    println!("MEDIA_DELIVERY_HARNESS_URL=http://{address}");
    println!("MEDIA_DELIVERY_START_PATH={start_path}");
    println!("MEDIA_DELIVERY_AUTHORITY=GatewayService::start_media_delivery");
    axum::serve(listener, service.router())
        .await
        .expect("serve media delivery harness");
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
