use axum::{
    Extension, Json,
    extract::{Path, State},
};
use tracing::instrument;

use shipyard_web::{ApiError, RequestId};

use crate::AppState;
use crate::http::v1::orders::repo;
use crate::http::v1::orders::types::GetOrderResponse;

#[instrument(
    name = "orders.get",
    skip(state),
    fields(request_id = %req_id.0, order_id = %id)
)]
pub async fn get_order(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    Path(id): Path<String>,
) -> Result<Json<GetOrderResponse>, ApiError> {
    let uuid = sqlx::types::Uuid::parse_str(&id)
        .map_err(|_| ApiError::validation(&req_id, "invalid id (expected UUID)"))?;

    let row = repo::get_order_by_id_tx(&state.db, &req_id, &uuid.to_string()).await?;

    Ok(Json(GetOrderResponse {
        id: row.id.to_string(),
        external_id: row.external_id,
        item_count: row.item_count,
        total_qty: row.total_qty,
    }))
}
