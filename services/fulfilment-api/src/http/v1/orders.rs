//! Orders routes for API v1.

use axum::{Extension, Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use shipyard_web::{ApiError, RequestId};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/validate", post(validate_order))
}

#[derive(Debug, Deserialize)]
pub struct ValidateOrderRequest {
    pub external_id: String,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Deserialize)]
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

fn validate(req: &ValidateOrderRequest, req_id: &RequestId) -> Result<(), ApiError> {
    if req.external_id.trim().is_empty() {
        return Err(ApiError::validation(
            req_id,
            "external_id must not be empty",
        ));
    }
    if req.items.is_empty() {
        return Err(ApiError::validation(req_id, "items must not be empty"));
    }

    for (idx, item) in req.items.iter().enumerate() {
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
