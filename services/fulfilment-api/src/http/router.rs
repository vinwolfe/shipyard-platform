//! Top-level router wiring (runtime endpoints + versioned API).

use axum::{Extension, Router, middleware::from_fn, routing::get};

use crate::AppState;
use crate::http::error::ApiError;
use crate::http::middleware::{RequestId, request_id};
use crate::http::v1;

pub fn build_router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ready" }))
        .nest("/api/v1", v1::router())
        .fallback(not_found)
        .layer(from_fn(request_id::middleware))
}

async fn not_found(Extension(req_id): Extension<RequestId>) -> ApiError {
    ApiError::not_found(&req_id)
}
