mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn healthz_returns_200() {
    let res = common::send("GET", "/healthz").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn readyz_returns_200() {
    let res = common::send("GET", "/readyz").await;
    assert_eq!(res.status(), StatusCode::OK);
}
