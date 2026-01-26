use crate::outbox::types::{OutboxEvent, OutboxRow, OutboxStatus};
use sqlx::{Postgres, Transaction};

/// Enqueue an outbox event inside the caller's transaction.
///
/// Must be called in the same transaction as the domain write.
pub async fn enqueue(
    tx: &mut Transaction<'_, Postgres>,
    event: &OutboxEvent,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO outbox (id, event_type, payload, status)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(event.id)
    .bind(&event.event_type)
    .bind(&event.payload)
    .bind(OutboxStatus::Pending.as_str())
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Claim up to `limit` pending rows that are available, using row locks.
///
/// Uses `FOR UPDATE SKIP LOCKED` to support multiple workers without double-processing.
/// Implemented as one atomic update to avoid N+1 updates.
pub async fn claim_batch(
    tx: &mut Transaction<'_, Postgres>,
    limit: i64,
) -> Result<Vec<OutboxRow>, sqlx::Error> {
    let rows: Vec<OutboxRow> = sqlx::query_as(
        r#"
        WITH picked AS (
          SELECT id
          FROM outbox
          WHERE status = 'PENDING' AND available_at <= now()
          ORDER BY created_at
          FOR UPDATE SKIP LOCKED
          LIMIT $1
        )
        UPDATE outbox o
        SET status = 'PROCESSING',
            locked_at = now(),
            updated_at = now()
        FROM picked
        WHERE o.id = picked.id
        RETURNING o.id, o.event_type, o.payload, o.status, o.attempts
        "#,
    )
    .bind(limit)
    .fetch_all(&mut **tx)
    .await?;

    Ok(rows)
}

pub async fn mark_sent(
    tx: &mut Transaction<'_, Postgres>,
    id: sqlx::types::Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE outbox
        SET status = 'SENT', updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Mark failed and schedule retry using backoff.
/// For now: linear backoff (attempts seconds). We'll refine later if needed.
pub async fn mark_failed(
    tx: &mut Transaction<'_, Postgres>,
    id: sqlx::types::Uuid,
    attempts: i32,
    err: &str,
) -> Result<(), sqlx::Error> {
    // Minimal backoff: available_at = now() + attempts seconds
    sqlx::query(
        r#"
        UPDATE outbox
        SET status = 'PENDING',
            attempts = attempts + 1,
            available_at = now() + make_interval(secs => GREATEST($2, 1)),
            last_error = $3,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(attempts)
    .bind(err)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
