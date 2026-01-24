mod common_db;

#[tokio::test]
#[ignore] // run via: make test-db
async fn db_readyz_is_200_with_db() {
    let app = common_db::app().await;
    let res = common_db::send(app.clone(), "GET", "/readyz").await;

    assert_eq!(res.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
#[ignore] // run via: make test-db
async fn db_create_and_get_order_roundtrip() {
    let app = common_db::app().await;

    let external_id = format!("ord_db_{}", uuid::Uuid::new_v4());
    let body = format!(
        r#"{{"external_id":"{}","items":[{{"sku":"ABC","qty":2}}]}}"#,
        external_id
    );
    let res = common_db::send_json(app.clone(), "POST", "/api/v1/orders", &body).await;
    assert_eq!(res.status(), axum::http::StatusCode::CREATED);

    let created = common_db::body_json(res).await;
    let id = created["id"].as_str().unwrap();

    let res = common_db::send(app.clone(), "GET", &format!("/api/v1/orders/{}", id)).await;
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    let got = common_db::body_json(res).await;
    assert_eq!(got["id"], id);
}
