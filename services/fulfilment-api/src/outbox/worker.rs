use sqlx::PgPool;
use std::time::Duration;

use super::delivery::DeliverySink;
use super::repo::{claim_batch, mark_failed, mark_sent};

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub poll_interval: Duration,
    pub batch_size: i64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(2),
            batch_size: 50,
        }
    }
}

/// Run one polling loop
pub async fn run(db: PgPool, sink: impl DeliverySink, cfg: WorkerConfig) -> anyhow::Result<()> {
    loop {
        match tick(&db, &sink, cfg.batch_size).await {
            Ok(()) => {}
            Err(e) if is_missing_outbox_table(&e) => {
                tracing::warn!(message = "outbox table missing; waiting for migrations");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                tracing::error!(error = %e, "outbox.worker.tick_failed");
                tokio::time::sleep(cfg.poll_interval).await;
            }
        }

        tokio::time::sleep(cfg.poll_interval).await;
    }
}

async fn tick(db: &PgPool, sink: &impl DeliverySink, batch_size: i64) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;
    let rows = claim_batch(&mut tx, batch_size).await?;
    tx.commit().await?;

    if rows.is_empty() {
        return Ok(());
    }

    for r in rows {
        let delivered = sink.deliver(&r.event_type, &r.payload).await;

        let mut tx = db.begin().await?;
        match delivered {
            Ok(()) => {
                mark_sent(&mut tx, r.id).await?;
            }
            Err(e) => {
                mark_failed(&mut tx, r.id, r.attempts, &e.message).await?;
            }
        }
        tx.commit().await?;
    }

    Ok(())
}

fn is_missing_outbox_table(err: &anyhow::Error) -> bool {
    // Postgres undefined_table = 42P01
    if let Some(sqlx::Error::Database(db_err)) = err.downcast_ref::<sqlx::Error>() {
        return db_err.code().as_deref() == Some("42P01");
    }
    false
}
