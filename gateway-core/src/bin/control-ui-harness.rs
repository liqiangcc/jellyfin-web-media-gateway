use gateway_core::GatewayService;
use std::env;
use std::net::IpAddr;
use tokio::net::TcpListener;
use url::Url;

#[tokio::main]
async fn main() {
    let host: IpAddr = env::var("CONTROL_UI_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1".into())
        .parse()
        .expect("CONTROL_UI_BIND_ADDR must be an IP address");
    let port: u16 = env::var("CONTROL_UI_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8787);
    let listener = TcpListener::bind((host, port))
        .await
        .expect("bind Control UI harness");
    let address = listener.local_addr().expect("Control UI harness address");
    let service = GatewayService::new(256);
    service
        .configure_http_authority(Url::parse(&format!("http://{address}")).unwrap())
        .expect("configure Control UI HTTP authority");
    let session_id = service
        .seed_control_ui_harness_session()
        .expect("seed Control UI harness session");
    println!("CONTROL_UI_HARNESS_URL=http://{address}/control?session_id={session_id}");
    println!("CONTROL_UI_HARNESS_SESSION_ID={session_id}");
    axum::serve(listener, service.router())
        .await
        .expect("serve Control UI harness");
}
