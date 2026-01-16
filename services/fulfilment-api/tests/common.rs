use axum::{Router, body::Body, http::Request, response::Response};
use tower::ServiceExt;

pub fn app() -> Router {
    fulfilment_api::build_app(shipyard_config::AppConfig::dev())
}

pub async fn send(method: &str, uri: &str) -> Response {
    app()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}
