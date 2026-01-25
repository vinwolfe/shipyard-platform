use shipyard_web::{ApiError, RequestId};

use super::types::{CreateOrderRequest, OrderItem, ValidateOrderRequest};

pub fn validate_order(req: &ValidateOrderRequest, req_id: &RequestId) -> Result<(), ApiError> {
    validate_items(&req.external_id, &req.items, req_id)
}

pub fn create_order(req: &CreateOrderRequest, req_id: &RequestId) -> Result<(), ApiError> {
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
