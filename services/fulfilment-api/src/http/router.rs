//! Top-level router composition.
//!
//! Why this file exists:
//! - Keeps `lib.rs` focused on app assembly, not route wiring.
//! - Provides a single place to mount versioned APIs (`/api/v1`) and service-level/runtime endpoints.
//! - Encourages a consistent routing shape as the service grows.

use crate::http::{
    error::ApiError,
    middleware::{RequestId, request_id},
    v1,
};
use axum::{Extension, Router, middleware::from_fn, routing::get};

/// Build the service router.
///
/// Design notes:
/// - `/healthz` and `/readyz` stay at the root because they are runtime contracts.
/// - Versioned (Business) API lives under `/api/v1` to avoid ad-hoc route sprawl.
pub fn build_router() -> Router {
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
