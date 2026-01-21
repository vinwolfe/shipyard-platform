//! HTTP metrics middleware.
//!
//! Records per-request Prometheus metrics using matched route patterns.
//! `/metrics` is excluded to avoid scrape noise.

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use std::time::Instant;

pub async fn middleware(req: Request, next: Next) -> Response {
    // NOTE: Own the route string so we can move `req` into `next.run(req)`.
    let route: String = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    // Exclude /metrics to avoid Prometheus scrape noise in the main HTTP metrics.
    if route == "/metrics" {
        return next.run(req).await;
    }

    let method = req.method().as_str().to_string();

    let start = Instant::now();
    let res = next.run(req).await;
    let dur = start.elapsed();

    let status = res.status().as_u16();

    crate::metrics::METRICS.record_http_request(&method, &route, status, dur);

    res
}
