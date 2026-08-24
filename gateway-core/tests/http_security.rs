use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use gateway_core::GatewayService;
use tower::ServiceExt;
use url::Url;

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

fn service() -> GatewayService {
    let service = GatewayService::new(8);
    service
        .configure_http_authority(Url::parse(ORIGIN).unwrap())
        .unwrap();
    service
}

#[tokio::test]
async fn http_surface_requires_valid_host_and_same_origin_for_cross_origin_reads() {
    let service = service();
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

    let attacker_host = Request::builder()
        .uri("/api/v1/display-probe/state")
        .header(header::HOST, "attacker.example")
        .header(header::ORIGIN, "http://attacker.example")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        service
            .router()
            .oneshot(attacker_host)
            .await
            .unwrap()
            .status(),
        StatusCode::MISDIRECTED_REQUEST
    );
}

#[tokio::test]
async fn probe_mutations_require_json_origin_and_bounded_body() {
    let service = service();

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
async fn authority_policy_binds_origin_scheme_and_default_port() {
    let service = GatewayService::new(8);
    service
        .configure_http_authority(Url::parse("http://gateway.example").unwrap())
        .unwrap();

    let valid = Request::builder()
        .uri("/control")
        .header(header::HOST, "gateway.example")
        .header(header::ORIGIN, "http://gateway.example:80")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        service.router().oneshot(valid).await.unwrap().status(),
        StatusCode::OK
    );

    for origin in [
        "https://gateway.example",
        "http://gateway.example:443",
        "http://gateway.example:81",
    ] {
        let request = Request::builder()
            .uri("/control")
            .header(header::HOST, "gateway.example")
            .header(header::ORIGIN, origin)
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            service.router().oneshot(request).await.unwrap().status(),
            StatusCode::FORBIDDEN,
            "origin authority must be bound to configured HTTP scheme/port: {origin}"
        );
    }
}

#[tokio::test]
async fn probe_telemetry_does_not_echo_header_secret_sentinels() {
    let service = service();
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
