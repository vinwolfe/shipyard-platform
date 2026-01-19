use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

fn app() -> Router {
    shipyard_web::apply_web_contract(Router::new().route("/ok", get(|| async { "ok" })))
}

fn req(uri: &str, req_id: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(id) = req_id {
        builder = builder.header("x-request-id", id);
    }
    builder.body(Body::empty()).expect("build request")
}

fn header_str<'a>(res: &'a axum::response::Response, name: &str) -> &'a str {
    res.headers()
        .get(name)
        .unwrap_or_else(|| panic!("missing header: {name}"))
        .to_str()
        .unwrap_or_else(|_| panic!("invalid header value: {name}"))
}

async fn json_body(res: axum::response::Response) -> Value {
    let bytes = res
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("parse json body")
}

#[tokio::test]
async fn reuses_inbound_x_request_id() {
    let res = app()
        .oneshot(req("/ok", Some("fixed-id-123")))
        .await
        .expect("oneshot");

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(header_str(&res, "x-request-id"), "fixed-id-123");
}

#[tokio::test]
async fn generates_x_request_id_when_missing() {
    let res = app().oneshot(req("/ok", None)).await.expect("oneshot");

    assert_eq!(res.status(), StatusCode::OK);
    assert!(!header_str(&res, "x-request-id").trim().is_empty());
}

#[tokio::test]
async fn unknown_route_returns_json_404_with_request_id() {
    let res = app()
        .oneshot(req("/does-not-exist", None))
        .await
        .expect("oneshot");

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert!(!header_str(&res, "x-request-id").trim().is_empty());

    let v = json_body(res).await;
    assert_eq!(v["error"]["code"], "NOT_FOUND");
    assert!(
        !v["error"]["request_id"]
            .as_str()
            .expect("error.request_id is string")
            .trim()
            .is_empty()
    );
}

#[tokio::test]
async fn contract_is_state_generic() {
    #[derive(Clone)]
    struct DummyState;

    let router: Router<DummyState> = Router::new().route("/ok", get(|| async { "ok" }));
    let app = shipyard_web::apply_web_contract(router).with_state(DummyState);

    let res = app.oneshot(req("/ok", None)).await.expect("oneshot");

    assert_eq!(res.status(), StatusCode::OK);
    assert!(!header_str(&res, "x-request-id").trim().is_empty());
}
