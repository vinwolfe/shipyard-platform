use std::net::SocketAddr;

use shipyard_config::AppConfig;

const SERVICE_NAME: &str = "fulfilment-api";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?; // fail fast

    let service_name = std::env::var("OTEL_SERVICE_NAME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| SERVICE_NAME.to_string());

    shipyard_observability::init(shipyard_observability::ObservabilityConfig {
        service_name: service_name.clone(),
        otlp_endpoint: config.otel_exporter_otlp_endpoint.clone(),
        log_filter: None,
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.service_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let app = fulfilment_api::build_app(config.clone());

    tracing::info!(addr = %addr, port = config.service_port, "{} listening", service_name);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("{} stopped", service_name);

    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut term = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        let mut int = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");

        tokio::select! {
            _ = term.recv() => {
                tracing::info!("shutdown signal received (SIGTERM)");
            }
            _ = int.recv() => {
                tracing::info!("shutdown signal received (SIGINT)");
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("shutdown signal received");
    }

    shipyard_observability::shutdown();
}
