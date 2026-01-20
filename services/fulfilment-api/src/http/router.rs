//! Top-level router wiring (runtime endpoints + versioned API).

use axum::http::header;
use axum::response::IntoResponse;
use axum::{Router, routing::get};

use crate::AppState;
use crate::http::v1;

pub fn build_router() -> Router<AppState> {
    shipyard_web::apply_web_contract(
        Router::new()
            .route("/healthz", get(|| async { "ok" }))
            .route("/readyz", get(|| async { "ready" }))
            .route("/metrics", get(metrics_stub))
            .nest("/api/v1", v1::router()),
    )
}

async fn metrics_stub() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        "# TODO: Prometheus metrics\n",
    )
}
