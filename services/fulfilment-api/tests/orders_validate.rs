use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

/// Valid payload should return 200.
///
/// TODO: Assert response schema fields once response conventions are standardised.
#[tokio::test]
async fn validate_order_valid_payload_returns_200() {
    let app = fulfilment_api::build_app();

    let body = r#"
    {
      "external_id": "ord_123",
      "items": [{"sku":"ABC","qty":1}]
    }
    "#;

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/orders/validate")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().get("x-request-id").is_some());
}

/// Invalid payload should return a client error (4xx).
#[tokio::test]
async fn validate_order_invalid_payload_returns_4xx() {
    let app = fulfilment_api::build_app();

    // empty external_id, empty items
    let body = r#"
    {
      "external_id": "",
      "items": []
    }
    "#;

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/orders/validate")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(res.status().is_client_error());
    assert!(res.headers().get("x-request-id").is_some());

    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&bytes).unwrap();

    let rid = v["error"]["request_id"].as_str().unwrap();
    assert!(!rid.is_empty());

    let code = v["error"]["code"].as_str().unwrap();
    assert!(!code.is_empty());
}

// Invalid route should return 404
#[tokio::test]
async fn unknown_route_returns_json_404_with_request_id() {
    let app = fulfilment_api::build_app();

    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert!(res.headers().get("x-request-id").is_some());

    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(v["error"]["code"], "NOT_FOUND");
    assert!(!v["error"]["request_id"].as_str().unwrap().is_empty());
}
