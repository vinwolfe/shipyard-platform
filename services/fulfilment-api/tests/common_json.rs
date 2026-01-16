use axum::{Router, body::Body, http::Request, response::Response};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

fn app() -> Router {
    fulfilment_api::build_app(shipyard_config::AppConfig::dev())
}

pub async fn send_json(method: &str, uri: &str, body: &str) -> Response {
    app()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

pub async fn body_json(res: Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}
