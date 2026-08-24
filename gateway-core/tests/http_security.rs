use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use gateway_core::GatewayService;
use tower::ServiceExt;

const HOST: &str = "127.0.0.1:8787";
const ORIGIN: &str = "http://127.0.0.1:8787";

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header(header::HOST, HOST)
        .body(Body::empty())
        .unwrap()
}

fn json_post(path: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header(header::HOST, HOST)
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

#[tokio::test]
async fn http_surface_requires_valid_host_and_same_origin_for_cross_origin_reads() {
    let service = GatewayService::new(8);
    for path in [
        "/control",
        "/display",
        "/api/v1/display-probe/state",
        "/api/v1/display-probe/events",
    ] {
        let missing_host = Request::builder().uri(path).body(Body::empty()).unwrap();
        assert_eq!(
            service
                .router()
                .oneshot(missing_host)
                .await
                .unwrap()
                .status(),
            StatusCode::BAD_REQUEST,
            "missing Host must be rejected for {path}"
        );
    }

    let malformed_host = Request::builder()
        .uri("/control")
        .header(header::HOST, "127.0.0.1/path")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        service
            .router()
            .oneshot(malformed_host)
            .await
            .unwrap()
            .status(),
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        service
            .router()
            .oneshot(get("/control"))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );

    let cross_origin = Request::builder()
        .uri("/api/v1/display-probe/state")
        .header(header::HOST, HOST)
        .header(header::ORIGIN, "https://attacker.example")
        .body(Body::empty())
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
}

#[tokio::test]
async fn probe_mutations_require_json_origin_and_bounded_body() {
    let service = GatewayService::new(8);

    let missing_origin = Request::builder()
        .method("POST")
        .uri("/api/v1/display-probe/reset")
        .header(header::HOST, HOST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(
        service
            .router()
            .oneshot(missing_origin)
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );

    let wrong_content_type = Request::builder()
        .method("POST")
        .uri("/api/v1/display-probe/commands")
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

    assert_eq!(
        service
            .router()
            .oneshot(json_post(
                "/api/v1/display-probe/commands",
                r#"{"request_id":"c10-command-1"}"#,
            ))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );

    let oversized = "x".repeat(40 * 1024);
    assert_eq!(
        service
            .router()
            .oneshot(json_post(
                "/api/v1/display-probe/telemetry",
                &format!(r#"{{"kind":"{oversized}"}}"#),
            ))
            .await
            .unwrap()
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn probe_telemetry_does_not_echo_header_secret_sentinels() {
    let service = GatewayService::new(8);
    let response = service
        .router()
        .oneshot(json_post(
            "/api/v1/display-probe/telemetry",
            r#"{"kind":"error","error_message":"Authorization Bearer c10-secret-alpha Cookie: c10-cookie-beta"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = service
        .router()
        .oneshot(get("/api/v1/display-probe/state"))
        .await
        .unwrap();
    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(!text.contains("c10-secret-alpha"));
    assert!(!text.contains("c10-cookie-beta"));
    assert!(text.contains("[redacted]"));
}
