use axum::{Router, extract::Extension, middleware::from_fn};
use opentelemetry::trace::TraceContextExt;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{ApiError, RequestId, request_id_middleware};

/// Apply the standard Shipyard web contract to a router.
///
/// Contract:
/// - `x-request-id` is always present on responses
/// - every request has a span carrying `request_id`, `trace_id`, `span_id`
/// - 404 returns standard JSON error envelope including request_id
pub fn apply_web_contract<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .fallback(not_found)
        .layer(trace_layer())
        .layer(from_fn(request_id_middleware))
}

fn trace_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    impl Fn(&axum::http::Request<axum::body::Body>) -> Span + Clone,
> {
    TraceLayer::new_for_http()
        .make_span_with(|req: &axum::http::Request<axum::body::Body>| {
            let req_id = req
                .extensions()
                .get::<RequestId>()
                .map(|r| r.0.as_str())
                .unwrap_or("");

            // Create the request span with empty placeholders for correlation fields.
            let span: Span = tracing::info_span!(
                "http.request",
                request_id = %req_id,
                method = %req.method(),
                path = %req.uri().path(),
            );

            record_trace_ids(&span);
            span
        })
        // Emit an INFO event when the request starts
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        // Emit an INFO event when the response is produced (includes status + latency)
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(tower_http::LatencyUnit::Micros),
        )
        // Emit an ERROR event when the request fails (5xx by default classifier)
        .on_failure(
            DefaultOnFailure::new()
                .level(Level::ERROR)
                .latency_unit(tower_http::LatencyUnit::Micros),
        )
}

/// Record OTEL trace/span ids onto the tracing span (best-effort).
fn record_trace_ids(span: &Span) {
    let cx = span.context();
    let otel_span = cx.span();
    let ctx = otel_span.span_context();

    if ctx.is_valid() {
        span.record("trace_id", tracing::field::display(ctx.trace_id()));
        span.record("span_id", tracing::field::display(ctx.span_id()));
    }
}

pub async fn not_found(Extension(req_id): Extension<RequestId>) -> ApiError {
    ApiError::not_found(&req_id)
}
