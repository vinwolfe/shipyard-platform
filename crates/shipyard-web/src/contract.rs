use axum::{Router, extract::Extension, middleware::from_fn};
use tower_http::trace::TraceLayer;

use crate::{ApiError, RequestId, request_id_middleware, request_log_middleware};

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
        // inside span: can read RequestId extension AND Span::current has OTEL context
        .layer(from_fn(request_log_middleware))
        // creates `http.request` span using RequestId extension
        .layer(trace_layer())
        // outermost: runs first, inserts RequestId into extensions + sets x-request-id header
        .layer(from_fn(request_id_middleware))
}

fn trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    impl Fn(&axum::http::Request<axum::body::Body>) -> tracing::Span + Clone,
> {
    TraceLayer::new_for_http().make_span_with(|req: &axum::http::Request<axum::body::Body>| {
        let req_id = req
            .extensions()
            .get::<RequestId>()
            .map(|r| r.0.as_str())
            .unwrap_or("");

        tracing::info_span!(
            "http.request",
            request_id = %req_id,
            method = %req.method(),
            path = %req.uri().path(),
        )
    })
}

pub async fn not_found(Extension(req_id): Extension<RequestId>) -> ApiError {
    ApiError::not_found(&req_id)
}
