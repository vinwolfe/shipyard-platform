//! Handler: POST /api/v1/orders/validate

use axum::{Extension, Json};
use tracing::instrument;

use shipyard_web::{ApiError, RequestId};

use crate::http::v1::orders::types::{
    NormalizedOrder, ValidateOrderRequest, ValidateOrderResponse,
};
use crate::http::v1::orders::validate as validators;

#[instrument(
    name = "orders.validate",
    skip(req, req_id),
    fields(request_id = %req_id.0, external_id = %req.external_id)
)]
pub async fn validate_order(
    Extension(req_id): Extension<RequestId>,
    Json(req): Json<ValidateOrderRequest>,
) -> Result<Json<ValidateOrderResponse>, ApiError> {
    // Shared validation (kept out of the handler to avoid duplication across endpoints).
    validators::validate_order(&req, &req_id)?;

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
