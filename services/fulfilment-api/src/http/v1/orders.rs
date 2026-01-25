//! Orders routes for API v1.
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

use shipyard_config::AppConfig;
use shipyard_web::{ApiError, RequestId};

use crate::AppState;

const IDEMPOTENCY_ENDPOINT: &str = "POST:/api/v1/orders";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/validate", post(validate_order))
        .route("/", post(create_order))
        .route("/:id", get(get_order))
}

pub fn router_no_db() -> Router<AppConfig> {
    Router::new().route("/validate", post(validate_order))
}

#[derive(Debug, Deserialize)]
pub struct ValidateOrderRequest {
    pub external_id: String,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderItem {
    pub sku: String,
    pub qty: i32,
}

#[derive(Debug, Serialize)]
pub struct ValidateOrderResponse {
    pub normalized: NormalizedOrder,
}

#[derive(Debug, Serialize)]
pub struct NormalizedOrder {
    pub external_id: String,
    pub item_count: usize,
    pub total_qty: i32,
}

// ===== Persistence DTOs =====

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateOrderRequest {
    pub external_id: String,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateOrderResponse {
    pub id: String,
    pub external_id: String,
    pub item_count: i32,
    pub total_qty: i32,
}

#[derive(Debug, Serialize)]
pub struct GetOrderResponse {
    pub id: String,
    pub external_id: String,
    pub item_count: i32,
    pub total_qty: i32,
}

#[derive(Debug, FromRow)]
struct OrderRow {
    id: sqlx::types::Uuid,
    external_id: String,
    item_count: i32,
    total_qty: i32,
}

// ===== Handlers =====

#[instrument(
    name = "orders.validate",
    skip(req, req_id),
    fields(request_id = %req_id.0, external_id = %req.external_id)
)]
async fn validate_order(
    Extension(req_id): Extension<RequestId>,
    Json(req): Json<ValidateOrderRequest>,
) -> Result<Json<ValidateOrderResponse>, ApiError> {
    validate(&req, &req_id)?;

    let item_count = req.items.len();
    let total_qty = req.items.iter().map(|i| i.qty).sum();

    Ok(Json(ValidateOrderResponse {
        normalized: NormalizedOrder {
            external_id: req.external_id,
            item_count,
            total_qty,
        },
    }))
}

async fn create_order_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
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

#[instrument(
    name = "orders.create",
    skip(state, headers, req),
    fields(request_id = %req_id.0, external_id = %req.external_id)
)]
async fn create_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(req_id): Extension<RequestId>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<CreateOrderResponse>), ApiError> {
    validate_create(&req, &req_id)?;

    let item_count: i32 = req.items.len() as i32;
    let total_qty: i32 = req.items.iter().map(|i| i.qty).sum();

    // Owned values captured by the idempotent op.
    let external_id = req.external_id.clone();
    let req_id_for_db = req_id.clone();

    crate::idempotency::with_idempotency(
        &state.db,
        &headers,
        &req_id, // borrow is fine
        IDEMPOTENCY_ENDPOINT,
        &req, // borrow is fine (hashing / conflict check)
        move |tx| {
            // Capture *clones*, not the borrowed req_id
            let external_id = external_id.clone();
            let req_id_for_db = req_id_for_db.clone();

            Box::pin(async move {
                let (status, body) =
                    create_order_tx(tx, &req_id_for_db, external_id, item_count, total_qty).await?;

                Ok((status, body))
            })
        },
    )
    .await
    .map(|(status, body)| (status, Json(body)))
}

#[instrument(
    name = "orders.get",
    skip(state),
    fields(request_id = %req_id.0, order_id = %id)
)]
async fn get_order(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    Path(id): Path<String>,
) -> Result<Json<GetOrderResponse>, ApiError> {
    let id = sqlx::types::Uuid::parse_str(&id)
        .map_err(|_| ApiError::validation(&req_id, "invalid id (expected UUID)"))?;

    let row: Option<OrderRow> = sqlx::query_as(
        r#"
        SELECT id, external_id, item_count, total_qty
        FROM orders
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| map_db_error(&req_id, e))?;

    let Some(row) = row else {
        return Err(ApiError::not_found(&req_id));
    };

    Ok(Json(GetOrderResponse {
        id: row.id.to_string(),
        external_id: row.external_id,
        item_count: row.item_count,
        total_qty: row.total_qty,
    }))
}

// ===== Helpers =====

fn validate(req: &ValidateOrderRequest, req_id: &RequestId) -> Result<(), ApiError> {
    validate_items(&req.external_id, &req.items, req_id)
}

fn validate_create(req: &CreateOrderRequest, req_id: &RequestId) -> Result<(), ApiError> {
    validate_items(&req.external_id, &req.items, req_id)
}

fn validate_items(
    external_id: &str,
    items: &[OrderItem],
    req_id: &RequestId,
) -> Result<(), ApiError> {
    if external_id.trim().is_empty() {
        return Err(ApiError::validation(
            req_id,
            "external_id must not be empty",
        ));
    }
    if items.is_empty() {
        return Err(ApiError::validation(req_id, "items must not be empty"));
    }
    for (idx, item) in items.iter().enumerate() {
        if item.sku.trim().is_empty() {
            return Err(ApiError::validation(
                req_id,
                format!("items[{idx}].sku must not be empty"),
            ));
        }
        if item.qty <= 0 {
            return Err(ApiError::validation(
                req_id,
                format!("items[{idx}].qty must be > 0"),
            ));
        }
    }
    Ok(())
}

fn map_db_error(req_id: &RequestId, err: sqlx::Error) -> ApiError {
    // log for operators (don't leak details to clients)
    tracing::error!(error = %err, "db error");

    if let sqlx::Error::Database(db_err) = &err {
        // postgres unique_violation
        if db_err.code().as_deref() == Some("23505") {
            return ApiError::conflict(req_id, "external_id already exists");
        }
    }

    ApiError::internal(req_id)
}
