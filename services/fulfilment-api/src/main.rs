use std::net::SocketAddr;

use shipyard_config::AppConfig;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?; // fail fast

    let addr = SocketAddr::from(([0, 0, 0, 0], config.service_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let app = fulfilment_api::build_app(config);

    println!("fulfilment-api listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    println!("shutdown signal received");
}
