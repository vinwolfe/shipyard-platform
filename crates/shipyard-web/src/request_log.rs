use axum::{extract::Request, middleware::Next, response::Response};
use opentelemetry::trace::TraceContextExt;
use std::time::Instant;
use tracing::{Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::RequestId;

/// Canonical request log with correlation fields.
///
/// Emits exactly one log event per request:
/// - request_id (from extensions)
/// - trace_id/span_id (from current OTEL context)
/// - method/path/status/latency
pub async fn request_log_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();

    let req_id = req
        .extensions()
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_default();

    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();

    let res = next.run(req).await;

    if path == "/metrics" {
        return res;
    }

    let latency = start.elapsed();

    let span = Span::current();
    let cx = span.context();
    let otel_span = cx.span();
    let sc = otel_span.span_context();

    let trace_id = if sc.is_valid() {
        sc.trace_id().to_string()
    } else {
        "".to_string()
    };
    let span_id = if sc.is_valid() {
        sc.span_id().to_string()
    } else {
        "".to_string()
    };

    tracing::event!(
        Level::INFO,
        request_id = %req_id,
        trace_id = %trace_id,
        span_id = %span_id,
        method = %method,
        path = %path,
        status = res.status().as_u16(),
        latency_us = latency.as_micros() as u64,
        "request.completed"
    );

    res
}
