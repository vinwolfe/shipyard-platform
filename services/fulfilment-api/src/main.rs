use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, time::Duration};

use shipyard_config::AppConfig;

const SERVICE_NAME: &str = "fulfilment-api";
const DB_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DB_MAX_CONNECTIONS: u32 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Config (fail fast)
    let config = AppConfig::from_env()?; // fail fast

    // Service identity (allow override, but keep a stable default)
    let service_name = std::env::var("OTEL_SERVICE_NAME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| SERVICE_NAME.to_string());

    // Observability
    shipyard_observability::init(shipyard_observability::ObservabilityConfig {
        service_name: service_name.clone(),
        otlp_endpoint: config.otel_exporter_otlp_endpoint.clone(),
        log_filter: None,
    });

    // Dependencies (DB)
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (e.g. via docker-compose)");

    let db = PgPoolOptions::new()
        .max_connections(DB_MAX_CONNECTIONS)
        .acquire_timeout(DB_CONNECT_TIMEOUT)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    // App assembly
    let app = fulfilment_api::build_app(config.clone(), db);

    // Serve
    let addr = SocketAddr::from(([0, 0, 0, 0], config.service_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

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
