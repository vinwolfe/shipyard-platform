//! Top-level router wiring (runtime endpoints + versioned API).

use axum::http::header;
use axum::response::IntoResponse;
use axum::{Router, middleware, routing::get};

use crate::AppState;
use crate::http::{middleware::http_metrics, v1};

pub fn build_router() -> Router<AppState> {
    let app = shipyard_web::apply_web_contract(
        Router::new()
            .route("/healthz", get(|| async { "ok" }))
            .route("/readyz", get(|| async { "ready" }))
            .route("/metrics", get(metrics))
            .nest("/api/v1", v1::router()),
    );

    // NOTE: route_layer runs after route matching, so MatchedPath is available.
    app.route_layer(middleware::from_fn(http_metrics::middleware))
}

async fn metrics() -> impl IntoResponse {
    let body = crate::metrics::METRICS.encode();
    (
        [(header::CONTENT_TYPE, crate::metrics::PROM_CONTENT_TYPE)],
        body,
    )
}
