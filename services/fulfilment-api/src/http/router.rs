//! Top-level router wiring (runtime endpoints + versioned API).

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::{Router, middleware, routing::get};

use shipyard_config::AppConfig;

use crate::AppState;
use crate::http::{middleware::http_metrics, v1};

pub fn build_router() -> Router<AppState> {
    let app = shipyard_web::apply_web_contract(
        Router::new()
            .route("/healthz", get(|| async { "ok" }))
            .route("/readyz", get(readyz))
            .route("/metrics", get(metrics))
            .nest("/api/v1", v1::router()),
    );

    // NOTE: route_layer runs after route matching, so MatchedPath is available.
    app.route_layer(middleware::from_fn(http_metrics::middleware))
}

/// A DB-free router used for fast tests.
/// - /healthz works
/// - /readyz returns 503 (because DB is not configured)
/// - /metrics works (still useful in tests)
pub fn build_router_no_db() -> Router<AppConfig> {
    let app = shipyard_web::apply_web_contract(
        Router::new()
            .route("/healthz", get(|| async { "ok" }))
            .route(
                "/readyz",
                get(|| async { (StatusCode::SERVICE_UNAVAILABLE, "not ready") }),
            )
            .route("/metrics", get(metrics))
            .nest("/api/v1", v1::router_no_db()),
    );

    app.route_layer(middleware::from_fn(http_metrics::middleware))
}

async fn readyz(State(state): State<AppState>) -> (StatusCode, &'static str) {
    let ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();

    if ok {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not ready")
    }
}

async fn metrics() -> impl IntoResponse {
    let body = crate::metrics::METRICS.encode();
    (
        [(header::CONTENT_TYPE, crate::metrics::PROM_CONTENT_TYPE)],
        body,
    )
}
