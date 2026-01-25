// services/fulfilment-api/src/http/v1/orders/repo.rs
use shipyard_web::{ApiError, RequestId};
use sqlx::{FromRow, PgPool, Postgres, Transaction};

use axum::http::StatusCode;

use super::types::CreateOrderResponse;

#[derive(Debug, FromRow)]
pub(crate) struct OrderRow {
    pub(crate) id: sqlx::types::Uuid,
    pub(crate) external_id: String,
    pub(crate) item_count: i32,
    pub(crate) total_qty: i32,
}

/// Create an order inside an existing transaction.
/// This is intentionally “repo-shaped”: SQL strings live here, not in handlers.
pub async fn create_order_tx(
    tx: &mut Transaction<'_, Postgres>,
    req_id: &RequestId,
    external_id: String,
    item_count: i32,
    total_qty: i32,
) -> Result<(StatusCode, CreateOrderResponse), ApiError> {
    let id = sqlx::types::Uuid::new_v4();

    let row: OrderRow = sqlx::query_as(
        r#"
        INSERT INTO orders (id, external_id, item_count, total_qty)
        VALUES ($1, $2, $3, $4)
        RETURNING id, external_id, item_count, total_qty
        "#,
    )
    .bind(id)
    .bind(&external_id)
    .bind(item_count)
    .bind(total_qty)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| map_db_error(req_id, e))?;

    Ok((
        StatusCode::CREATED,
        CreateOrderResponse {
            id: row.id.to_string(),
            external_id: row.external_id,
            item_count: row.item_count,
            total_qty: row.total_qty,
        },
    ))
}

/// Fetch an order by id (UUID string).
pub async fn get_order_by_id_tx(
    db: &PgPool,
    req_id: &RequestId,
    id: &str,
) -> Result<OrderRow, ApiError> {
    let id = sqlx::types::Uuid::parse_str(id)
        .map_err(|_| ApiError::validation(req_id, "invalid id (expected UUID)"))?;

    let row: Option<OrderRow> = sqlx::query_as(
        r#"
        SELECT id, external_id, item_count, total_qty
        FROM orders
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
    .map_err(|e| map_db_error(req_id, e))?;

    row.ok_or_else(|| ApiError::not_found(req_id))
}

fn map_db_error(req_id: &RequestId, err: sqlx::Error) -> ApiError {
    // Log for operators (don’t leak details to clients).
    tracing::error!(error = %err, "db error");

    if let sqlx::Error::Database(db_err) = &err {
        // Postgres unique_violation
        if db_err.code().as_deref() == Some("23505") {
            return ApiError::conflict(req_id, "external_id already exists");
        }
    }

    ApiError::internal(req_id)
}
