// ===== API DTOs =====

use serde::{Deserialize, Serialize};

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
