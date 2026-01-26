use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

use fulfilment_api::outbox::{delivery::LogSink, worker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    tracing_subscriber::fmt()
        .json()
        .with_target(true)
        .with_current_span(true)
        .init();

    tracing::info!(message = "outbox-worker starting");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let cfg = worker::WorkerConfig {
        poll_interval: Duration::from_secs(2),
        batch_size: 50,
    };

    worker::run(db, LogSink, cfg).await
}
