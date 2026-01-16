mod common;
mod common_json;

use axum::http::StatusCode;

#[tokio::test]
async fn validate_order_valid_payload_returns_200() {
    let body = r#"{"external_id":"ord_123","items":[{"sku":"ABC","qty":1}]}"#;

    let res = common_json::send_json("POST", "/api/v1/orders/validate", body).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key("x-request-id"));
}

/// Invalid payload should return a client error (4xx).
#[tokio::test]
async fn validate_order_invalid_payload_returns_4xx() {
    let body = r#"{"external_id":"","items":[]}"#;

    let res = common_json::send_json("POST", "/api/v1/orders/validate", body).await;

    assert!(res.status().is_client_error());
    assert!(res.headers().contains_key("x-request-id"));

    let v = common_json::body_json(res).await;

    let rid = v["error"]["request_id"].as_str().unwrap();
    assert!(!rid.is_empty());

    let code = v["error"]["code"].as_str().unwrap();
    assert!(!code.is_empty());
}

// Invalid route should return 404
#[tokio::test]
async fn unknown_route_returns_404_with_request_id_and_not_found_code() {
    let res = common::send("GET", "/api/v1/does-not-exist").await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert!(res.headers().contains_key("x-request-id"));

    let v = common_json::body_json(res).await;

    assert_eq!(v["error"]["code"], "NOT_FOUND");
    assert!(!v["error"]["request_id"].as_str().unwrap().is_empty());
}
