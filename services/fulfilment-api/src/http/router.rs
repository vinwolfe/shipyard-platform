//! Top-level router wiring (runtime endpoints + versioned API).

use axum::{Router, routing::get};

use crate::AppState;
use crate::http::v1;

pub fn build_router() -> Router<AppState> {
    shipyard_web::apply_web_contract(
        Router::new()
            .route("/healthz", get(|| async { "ok" }))
            .route("/readyz", get(|| async { "ready" }))
            .nest("/api/v1", v1::router()),
    )
}
