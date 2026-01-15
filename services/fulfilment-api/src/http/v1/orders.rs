//! Orders routes for API v1.
//!
//! Notes:
//! - Early on, colocating routes + DTOs + handler keeps the change surface small.
//! - We extract shared utilities only when justified by reuse (thin platform principle).
//!
//! TODO: Move stable cross-cutting helpers (once proven) into shared HTTP utilities.
use crate::http::{error::ApiError, middleware::RequestId};
use axum::{Extension, Json, Router, routing::post};
use serde::{Deserialize, Serialize};

/// Orders router.
pub fn router() -> Router {
    Router::new().route("/validate", post(validate_order))
}

/// Request DTO for order validation.
#[derive(Debug, Deserialize)]
pub struct ValidateOrderRequest {
    pub external_id: String,
    pub items: Vec<OrderItem>,
}

/// Item DTO within the order.
#[derive(Debug, Deserialize)]
pub struct OrderItem {
    pub sku: String,
    pub qty: i32,
}

/// Response DTO for order validation.
#[derive(Debug, Serialize)]
pub struct ValidateOrderResponse {
    pub ok: bool,
    pub normalized: NormalizedOrder,
}

/// Normalized order DTO.
#[derive(Debug, Serialize)]
pub struct NormalizedOrder {
    pub external_id: String,
    pub item_count: usize,
    pub total_qty: i32,
}

/// Handler for validating an order payload.
async fn validate_order(
    Extension(req_id): Extension<RequestId>,
    Json(req): Json<ValidateOrderRequest>,
) -> Result<Json<ValidateOrderResponse>, ApiError> {
    // Deterministic validation rules (simple + explicit)
    //
    // TODO: If validation rules become complex or widely reused, introduce a validation approach
    // that remains explicit and testable (avoid magic-heavy frameworks).
    if req.external_id.trim().is_empty() {
        return Err(ApiError::validation(
            &req_id,
            "external_id must not be empty",
        ));
    }
    if req.items.is_empty() {
        return Err(ApiError::validation(&req_id, "items must not be empty"));
    }
    for (idx, item) in req.items.iter().enumerate() {
        if item.sku.trim().is_empty() {
            return Err(ApiError::validation(
                &req_id,
                format!("items[{idx}].sku must not be empty"),
            ));
        }
        if item.qty <= 0 {
            return Err(ApiError::validation(
                &req_id,
                format!("items[{idx}].qty must be > 0"),
            ));
        }
    }

    let item_count = req.items.len();
    let total_qty: i32 = req.items.iter().map(|i| i.qty).sum();

    let resp = ValidateOrderResponse {
        ok: true,
        normalized: NormalizedOrder {
            external_id: req.external_id,
            item_count,
            total_qty,
        },
    };

    Ok(Json(resp))
}
