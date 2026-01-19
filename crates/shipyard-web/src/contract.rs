use axum::{Router, extract::Extension, middleware::from_fn};

use crate::{ApiError, RequestId, request_id_middleware};

/// Apply the standard Shipyard web contract to a router.
///
/// Contract:
/// - `x-request-id` is always present on responses
/// - 404 returns standard JSON error envelope including request_id
pub fn apply_web_contract<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .fallback(not_found)
        .layer(from_fn(request_id_middleware))
}

/// Standard 404 handler that returns JSON error with request_id.
pub async fn not_found(Extension(req_id): Extension<RequestId>) -> ApiError {
    ApiError::not_found(&req_id)
}
