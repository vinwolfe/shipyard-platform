mod common_db;

use axum::http::StatusCode;

const IDEMPOTENCY_HEADER: &str = "Idempotency-Key";

#[tokio::test]
#[ignore] // run via: make test-db
async fn db_idempotency_same_key_same_payload_replays_same_response() {
    let app = common_db::app().await;

    let external_id = format!("ord_idem_{}", uuid::Uuid::new_v4());
    let payload = format!(
        r#"{{"external_id":"{}","items":[{{"sku":"ABC","qty":2}}]}}"#,
        external_id
    );

    let key = format!("idem_{}", uuid::Uuid::new_v4());

    let res1 = common_db::send_json_with_headers(
        app.clone(),
        "POST",
        "/api/v1/orders",
        &payload,
        &[(IDEMPOTENCY_HEADER, key.as_str())],
    )
    .await;

    if res1.status() == StatusCode::INTERNAL_SERVER_ERROR {
        let err = common_db::body_json(res1).await;
        eprintln!("500 body: {err}");
        panic!("unexpected 500");
    }
    assert_eq!(res1.status(), StatusCode::CREATED);
    let body1 = common_db::body_json(res1).await;

    let res2 = common_db::send_json_with_headers(
        app.clone(),
        "POST",
        "/api/v1/orders",
        &payload,
        &[(IDEMPOTENCY_HEADER, key.as_str())],
    )
    .await;

    if res2.status() == StatusCode::INTERNAL_SERVER_ERROR {
        let err = common_db::body_json(res2).await;
        eprintln!("500 body: {err}");
        panic!("unexpected 500");
    }
    assert_eq!(res2.status(), StatusCode::CREATED);
    let body2 = common_db::body_json(res2).await;

    // Strongest contract check: response body is identical (same id).
    assert_eq!(body1, body2);
    assert_eq!(body1["external_id"], external_id);
    assert!(body1["id"].as_str().is_some());
}

#[tokio::test]
#[ignore] // run via: make test-db
async fn db_idempotency_same_key_different_payload_returns_409() {
    let app = common_db::app().await;

    let external_id = format!("ord_idem_conflict_{}", uuid::Uuid::new_v4());
    let key = format!("idem_conflict_{}", uuid::Uuid::new_v4());

    let payload_a = format!(
        r#"{{"external_id":"{}","items":[{{"sku":"ABC","qty":1}}]}}"#,
        external_id
    );
    let payload_b = format!(
        r#"{{"external_id":"{}","items":[{{"sku":"ABC","qty":2}}]}}"#,
        external_id
    );

    let res1 = common_db::send_json_with_headers(
        app.clone(),
        "POST",
        "/api/v1/orders",
        &payload_a,
        &[(IDEMPOTENCY_HEADER, key.as_str())],
    )
    .await;
    if res1.status() == StatusCode::INTERNAL_SERVER_ERROR {
        let err = common_db::body_json(res1).await;
        eprintln!("500 body: {err}");
        panic!("unexpected 500");
    }
    assert_eq!(res1.status(), StatusCode::CREATED);

    let res2 = common_db::send_json_with_headers(
        app.clone(),
        "POST",
        "/api/v1/orders",
        &payload_b,
        &[(IDEMPOTENCY_HEADER, key.as_str())],
    )
    .await;

    if res2.status() == StatusCode::INTERNAL_SERVER_ERROR {
        let err = common_db::body_json(res2).await;
        eprintln!("500 body: {err}");
        panic!("unexpected 500");
    }
    assert_eq!(res2.status(), StatusCode::CONFLICT);

    // Basic envelope check (don’t overfit to message wording).
    let err = common_db::body_json(res2).await;
    assert!(err["error"]["request_id"].as_str().is_some());
    assert!(err["error"]["code"].as_str().is_some());
}

#[tokio::test]
#[ignore] // run via: make test-db
async fn db_no_idempotency_key_creates_two_distinct_orders() {
    let app = common_db::app().await;

    // Use different external_id to avoid your unique constraint conflict behaviour.
    let payload1 = format!(
        r#"{{"external_id":"ord_noidem_{}","items":[{{"sku":"ABC","qty":1}}]}}"#,
        uuid::Uuid::new_v4()
    );
    let payload2 = format!(
        r#"{{"external_id":"ord_noidem_{}","items":[{{"sku":"ABC","qty":1}}]}}"#,
        uuid::Uuid::new_v4()
    );

    let res1 = common_db::send_json(app.clone(), "POST", "/api/v1/orders", &payload1).await;
    if res1.status() == StatusCode::INTERNAL_SERVER_ERROR {
        let err = common_db::body_json(res1).await;
        eprintln!("500 body: {err}");
        panic!("unexpected 500");
    }
    assert_eq!(res1.status(), StatusCode::CREATED);
    let b1 = common_db::body_json(res1).await;

    let res2 = common_db::send_json(app.clone(), "POST", "/api/v1/orders", &payload2).await;
    assert_eq!(res2.status(), StatusCode::CREATED);
    let b2 = common_db::body_json(res2).await;

    let id1 = b1["id"].as_str().unwrap();
    let id2 = b2["id"].as_str().unwrap();
    assert_ne!(id1, id2);
}
