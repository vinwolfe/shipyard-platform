use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

/// Valid payload should return 200.
///
/// TODO: Assert response schema fields once response conventions are standardised.
/// TODO: Add request_id assertions once request correlation is introduced.
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
}

/// Invalid payload should return a client error (4xx).
///
/// TODO: Assert on the standard error schema once introduced (code/message/request_id).
/// TODO: Assert `x-request-id` exists on error responses once request correlation is introduced.
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
}
