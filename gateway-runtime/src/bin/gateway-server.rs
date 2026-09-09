use gateway_runtime::{GatewayRuntimeConfig, build_service};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GatewayRuntimeConfig::from_env()?;
    let service = build_service(&config)?;
    let listener = TcpListener::bind((config.bind_addr, config.port)).await?;
    let address = listener.local_addr()?;
    eprintln!("gateway-server listening on {address}");
    axum::serve(listener, service.router()).await?;
    Ok(())
}
