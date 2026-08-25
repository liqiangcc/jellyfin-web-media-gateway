use crate::{
    ControlCommand, ControlCommandRequest, ControlEventKind, ControlService, GatewayService,
};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Barrier;
use tower::ServiceExt;
use url::Url;

const HOST: &str = "127.0.0.1:8787";
const ORIGIN: &str = "http://127.0.0.1:8787";

fn service() -> GatewayService {
    let service = GatewayService::new(8);
    service
        .configure_http_authority(Url::parse(ORIGIN).unwrap())
        .unwrap();
    service
}

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header(header::HOST, HOST)
        .body(Body::empty())
        .unwrap()
}

fn json_post(path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header(header::HOST, HOST)
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    serde_json::from_slice(&body)
        .unwrap_or_else(|_| panic!("response was not JSON: {}", String::from_utf8_lossy(&body)))
}

fn command(request_id: &str, expected: u64, command: Value) -> Value {
    json!({
        "request_id": request_id,
        "expected_session_revision": expected,
        "command": command,
    })
}

#[test]
fn public_control_surface_has_no_caller_selected_media_seed_api() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let control_source = std::fs::read_to_string(manifest.join("src/control.rs")).unwrap();
    let lib_source = std::fs::read_to_string(manifest.join("src/lib.rs")).unwrap();
    assert!(!control_source.contains("pub fn seed_test_session("));
    assert!(control_source.contains("pub(crate) fn seed_test_session("));
    assert!(!lib_source.contains("pub fn control(&self)"));
}

#[tokio::test]
async fn sessions_are_isolated_and_snapshot_is_canonical_non_secret_view() {
    let service = service();
    let control = service.control();
    let first =
        control.seed_test_session("item-a", "https://cdn.invalid/raw-secret-a", "display-a");
    let second =
        control.seed_test_session("item-b", "https://cdn.invalid/raw-secret-b", "display-b");

    let first_snapshot = json_body(
        service
            .router()
            .oneshot(get(&format!("/api/v1/sessions/{first}")))
            .await
            .unwrap(),
    )
    .await;
    let second_snapshot = json_body(
        service
            .router()
            .oneshot(get(&format!("/api/v1/sessions/{second}")))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(first_snapshot["session_id"], first);
    assert_eq!(first_snapshot["current_item"]["item_id"], "item-a");
    assert_eq!(first_snapshot["session_revision"], 0);
    assert_eq!(second_snapshot["current_item"]["item_id"], "item-b");
    let text = format!("{first_snapshot}{second_snapshot}");
    assert!(!text.contains("raw-secret-a"));
    assert!(!text.contains("raw-secret-b"));
    assert!(!text.contains("resolved_media"));

    let missing = service
        .router()
        .oneshot(get("/api/v1/sessions/s-missing"))
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(missing).await["code"], "SESSION_NOT_FOUND");
}

#[tokio::test]
async fn http_commands_preserve_r007_idempotency_cas_and_event_semantics() {
    let service = service();
    let session_id = service
        .control()
        .seed_test_session("item-a", "media-secret", "display-a");
    let path = format!("/api/v1/sessions/{session_id}/commands");
    for (request_id, expected, body) in [
        ("play-1", 0, json!({"type":"play"})),
        ("pause-1", 1, json!({"type":"pause"})),
        ("seek-1", 2, json!({"type":"seek", "position_ms": 1200})),
        ("stop-1", 3, json!({"type":"stop"})),
        (
            "handoff-1",
            4,
            json!({"type":"begin_handoff", "target_display_id":"display-b"}),
        ),
    ] {
        assert_eq!(
            service
                .router()
                .oneshot(json_post(&path, command(request_id, expected, body)))
                .await
                .unwrap()
                .status(),
            StatusCode::OK,
            "{request_id}"
        );
    }

    let duplicate = service
        .router()
        .oneshot(json_post(
            &path,
            command(
                "handoff-1",
                4,
                json!({"type":"begin_handoff", "target_display_id":"display-b"}),
            ),
        ))
        .await
        .unwrap();
    let duplicate_body = json_body(duplicate).await;
    assert_eq!(duplicate_body["status"], "accepted");
    assert_eq!(duplicate_body["session_revision"], 5);
    assert_eq!(duplicate_body["event_cursor"], 5);

    let mismatch = service
        .router()
        .oneshot(json_post(
            &path,
            command("handoff-1", 5, json!({"type":"stop"})),
        ))
        .await
        .unwrap();
    assert_eq!(mismatch.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(mismatch).await["code"], "REQUEST_ID_MISMATCH");

    let stale = service
        .router()
        .oneshot(json_post(
            &path,
            command("stale-1", 4, json!({"type":"seek", "position_ms": 99})),
        ))
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::CONFLICT);
    let stale_body = json_body(stale).await;
    assert_eq!(stale_body["code"], "REVISION_CONFLICT");
    assert_eq!(stale_body["current_revision"], 5);

    let events = service
        .router()
        .oneshot(get(&format!(
            "/api/v1/sessions/{session_id}/events?after=0"
        )))
        .await
        .unwrap();
    let events_body = json_body(events).await;
    assert_eq!(events_body["cursor"], 5);
    assert_eq!(events_body["events"].as_array().unwrap().len(), 5);
    assert_eq!(
        events_body["events"][4]["snapshot"]["handoff"]["target_display_id"],
        "display-b"
    );
}

#[tokio::test]
async fn two_controls_through_http_cannot_commit_against_one_old_revision() {
    let service = service();
    let session_id = service
        .control()
        .seed_test_session("item-a", "media-a", "display-a");
    let path = format!("/api/v1/sessions/{session_id}/commands");
    let barrier = Arc::new(Barrier::new(3));

    let first_router = service.router();
    let first_barrier = Arc::clone(&barrier);
    let first_path = path.clone();
    let first = tokio::spawn(async move {
        first_barrier.wait().await;
        first_router
            .oneshot(json_post(
                &first_path,
                command("control-a", 0, json!({"type":"pause"})),
            ))
            .await
            .unwrap()
    });

    let second_router = service.router();
    let second_barrier = Arc::clone(&barrier);
    let second = tokio::spawn(async move {
        second_barrier.wait().await;
        second_router
            .oneshot(json_post(
                &path,
                command(
                    "control-b",
                    0,
                    json!({"type":"seek", "position_ms": 25_000}),
                ),
            ))
            .await
            .unwrap()
    });

    barrier.wait().await;
    let (first, second) = tokio::join!(first, second);
    let statuses = [first.unwrap().status(), second.unwrap().status()];
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::OK)
            .count(),
        1
    );
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::CONFLICT)
            .count(),
        1
    );
    let snapshot = control_snapshot(&service, &session_id).await;
    assert_eq!(snapshot["session_revision"], 1);
}

async fn control_snapshot(service: &GatewayService, session_id: &str) -> Value {
    json_body(
        service
            .router()
            .oneshot(get(&format!("/api/v1/sessions/{session_id}")))
            .await
            .unwrap(),
    )
    .await
}

#[tokio::test]
async fn telemetry_updates_observation_without_command_revision_and_rejects_stale_callbacks() {
    let service = service();
    let control = service.control();
    let session_id = control.seed_test_session("item-a", "media-a", "display-a");
    assert!(
        control
            .apply_position_telemetry(&session_id, "item-a", 1, 1, 10_000)
            .unwrap()
    );
    assert!(
        !control
            .apply_position_telemetry(&session_id, "item-a", 1, 1, 20_000)
            .unwrap()
    );
    assert!(
        !control
            .apply_position_telemetry(&session_id, "old-item", 1, 2, 30_000)
            .unwrap()
    );

    let snapshot = control.snapshot(&session_id).unwrap();
    assert_eq!(snapshot.session_revision, 0);
    assert_eq!(snapshot.position_ms, 10_000);
    assert_eq!(snapshot.telemetry_sequence, 1);
    let events = control.events_after(&session_id, 0).unwrap();
    assert_eq!(events.events.len(), 1);
    assert_eq!(events.events[0].kind, ControlEventKind::PositionTelemetry);
}

#[test]
fn reconnect_requires_snapshot_when_cursor_history_is_expired() {
    let control = ControlService::new(2);
    let session_id = control.seed_test_session("item-a", "media-a", "display-a");
    for (index, expected) in (0..3).enumerate() {
        control
            .execute_command(
                &session_id,
                ControlCommandRequest {
                    request_id: format!("req-{index}"),
                    expected_session_revision: Some(expected),
                    command: ControlCommand::Pause,
                },
            )
            .unwrap();
    }
    let expired = control.events_after(&session_id, 0).unwrap();
    assert!(expired.snapshot_required);
    assert_eq!(expired.reason, Some("cursor_expired"));
    let reconnect = control.events_after(&session_id, 1).unwrap();
    assert!(!reconnect.snapshot_required);
    assert_eq!(reconnect.events.len(), 2);
    assert_eq!(reconnect.events[0].cursor, 2);
    assert_eq!(reconnect.events[1].cursor, 3);
    assert_eq!(control.snapshot(&session_id).unwrap().session_revision, 3);
    assert!(
        control
            .events_after(&session_id, 4)
            .unwrap()
            .snapshot_required
    );
}

#[tokio::test]
async fn command_input_is_bounded_structured_and_does_not_accept_url_authority() {
    let service = service();
    let session_id = service
        .control()
        .seed_test_session("item-a", "media-a", "display-a");
    let path = format!("/api/v1/sessions/{session_id}/commands");

    let invalid_shape = service
        .router()
        .oneshot(json_post(
            &path,
            json!({
                "request_id":"url-test",
                "expected_session_revision":0,
                "command":{"type":"pause"},
                "url":"http://169.254.169.254/latest/meta-data"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(invalid_shape.status(), StatusCode::BAD_REQUEST);

    let invalid_id = service
        .router()
        .oneshot(json_post(
            &path,
            command("bad/id", 0, json!({"type":"pause"})),
        ))
        .await
        .unwrap();
    assert_eq!(invalid_id.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(invalid_id).await["code"], "REQUEST_ID_INVALID");

    let wrong_content_type = Request::builder()
        .method("POST")
        .uri(&path)
        .header(header::HOST, HOST)
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(
        service
            .router()
            .oneshot(wrong_content_type)
            .await
            .unwrap()
            .status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );

    let oversized = "x".repeat(40 * 1024);
    let oversized_request = Request::builder()
        .method("POST")
        .uri(&path)
        .header(header::HOST, HOST)
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(oversized))
        .unwrap();
    assert_eq!(
        service
            .router()
            .oneshot(oversized_request)
            .await
            .unwrap()
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn web_display_http_registration_lease_context_and_callback_are_generation_safe() {
    let service = service();
    let session_id = service
        .control()
        .seed_test_session("item-a", "media-not-public", "display-a");
    let registered = json_body(
        service
            .router()
            .oneshot(json_post(
                "/api/v1/displays/register",
                json!({
                    "session_id": session_id,
                    "display_id": "display-a",
                    "label": "Living room",
                    "capabilities": ["video", "audio"]
                }),
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(registered["page_lease_epoch"], 1);
    assert_ne!(registered["registration_id"], "display-a");
    let lease = registered["lease_token"].as_str().unwrap().to_owned();
    let context = &registered["context"];
    assert_eq!(context["display_id"], "display-a");
    assert_eq!(context["display_generation"], 1);

    let context_response = service
        .router()
        .oneshot(
            Request::builder()
                .uri("/api/v1/displays/display-a/context")
                .header(header::HOST, HOST)
                .header("x-display-lease", &lease)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(context_response.status(), StatusCode::OK);

    let callback_body = json!({
        "lease_token": lease,
        "session_id": context["session_id"],
        "item_id": context["item_id"],
        "item_revision": context["item_revision"],
        "session_revision": context["session_revision"],
        "display_id": context["display_id"],
        "display_generation": context["display_generation"],
        "telemetry_sequence": 1,
        "position_ms": 12000,
        "observation": "playing"
    });
    let callback_response = service
        .router()
        .oneshot(json_post(
            "/api/v1/displays/display-a/callback",
            callback_body.clone(),
        ))
        .await
        .unwrap();
    assert_eq!(callback_response.status(), StatusCode::OK);
    let callback_result = json_body(callback_response).await;
    assert!(callback_result["accepted"].as_bool().unwrap());
    assert_eq!(callback_result["telemetry_accepted"], true);

    let stale_item = service
        .router()
        .oneshot(json_post(
            "/api/v1/displays/display-a/callback",
            json!({
                "lease_token": registered["lease_token"],
                "session_id": context["session_id"],
                "item_id": "old-item",
                "item_revision": context["item_revision"],
                "session_revision": context["session_revision"],
                "display_id": context["display_id"],
                "display_generation": context["display_generation"],
                "telemetry_sequence": 2,
                "position_ms": 30000,
                "observation": "playing"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(stale_item.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(stale_item).await["code"], "STALE_DISPLAY_CONTEXT");
    assert_eq!(
        service.control().snapshot(&session_id).unwrap().position_ms,
        12000
    );

    let reconnected = json_body(
        service
            .router()
            .oneshot(json_post(
                "/api/v1/displays/register",
                json!({
                    "session_id": session_id,
                    "display_id": "display-a",
                    "label": "Living room refreshed",
                    "capabilities": ["video"],
                    "previous_registration_id": registered["registration_id"],
                    "previous_lease_token": registered["lease_token"]
                }),
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(reconnected["page_lease_epoch"], 2);
    let old_callback = service
        .router()
        .oneshot(json_post(
            "/api/v1/displays/display-a/callback",
            callback_body,
        ))
        .await
        .unwrap();
    assert_eq!(old_callback.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        service.control().snapshot(&session_id).unwrap().position_ms,
        12000
    );
}

#[tokio::test]
async fn web_display_http_routes_reuse_origin_and_body_security() {
    let service = service();
    let cross_origin = Request::builder()
        .method("POST")
        .uri("/api/v1/displays/register")
        .header(header::HOST, HOST)
        .header(header::ORIGIN, "https://attacker.example")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(
        service
            .router()
            .oneshot(cross_origin)
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );

    let oversized = Request::builder()
        .method("POST")
        .uri("/api/v1/displays/register")
        .header(header::HOST, HOST)
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("x".repeat(40 * 1024)))
        .unwrap();
    assert_eq!(
        service.router().oneshot(oversized).await.unwrap().status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}
