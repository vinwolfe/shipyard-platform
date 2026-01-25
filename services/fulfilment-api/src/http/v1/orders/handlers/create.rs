//! Handler: POST /api/v1/orders
//!
//! Notes:
//! - Uses Idempotency-Key (if provided) to guarantee safe retries.
//! - DB work is delegated to orders::repo to keep this handler thin.

use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use tracing::instrument;

use shipyard_web::{ApiError, RequestId};

use crate::AppState;
use crate::http::v1::orders::repo;
use crate::http::v1::orders::types::{CreateOrderRequest, CreateOrderResponse};
use crate::http::v1::orders::validate as validators;

const IDEMPOTENCY_ENDPOINT: &str = "POST:/api/v1/orders";

#[instrument(
    name = "orders.create",
    skip(state, headers, req),
    fields(request_id = %req_id.0, external_id = %req.external_id)
)]
pub async fn create_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(req_id): Extension<RequestId>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<CreateOrderResponse>), ApiError> {
    validators::create_order(&req, &req_id)?;

    // Precompute derived values outside the tx.
    let item_count: i32 = req.items.len() as i32;
    let total_qty: i32 = req.items.iter().map(|i| i.qty).sum();

    // Owned values for the idempotent op closure.
    let external_id = req.external_id.clone();
    let req_id_for_db = req_id.clone();

    crate::idempotency::with_idempotency(
        &state.db,
        &headers,
        &req_id,
        IDEMPOTENCY_ENDPOINT,
        &req, // used for request hashing / conflict detection
        move |tx| {
            // Clone for this invocation (closure may run once, but keep it simple).
            let external_id = external_id.clone();
            let req_id_for_db = req_id_for_db.clone();

            Box::pin(async move {
                // Delegate DB insert to repo module; keeps handler small and testable.
                repo::create_order_tx(tx, &req_id_for_db, external_id, item_count, total_qty).await
            })
        },
    )
    .await
    .map(|(status, body)| (status, Json(body)))
}
