// tests/outbox.rs
mod common_db;

use axum::http::StatusCode;

const IDEMPOTENCY_HEADER: &str = "Idempotency-Key";

#[tokio::test]
#[ignore] // run via: make test-db
async fn db_order_create_enqueues_outbox_row() {
    let app = common_db::app().await;

    let external_id = format!("ord_outbox_{}", uuid::Uuid::new_v4());
    let payload = format!(
        r#"{{"external_id":"{}","items":[{{"sku":"ABC","qty":1}}]}}"#,
        external_id
    );

    let res = common_db::send_json(app.clone(), "POST", "/api/v1/orders", &payload).await;

    if res.status() == StatusCode::INTERNAL_SERVER_ERROR {
        let err = common_db::body_json(res).await;
        eprintln!("500 body: {err}");
        panic!("unexpected 500");
    }

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = common_db::body_json(res).await;
    let order_id = body["id"].as_str().expect("response must include id");

    // Assert: exactly one outbox row exists for THIS order_id
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM outbox
        WHERE event_type = 'order.created'
          AND payload->>'order_id' = $1
        "#,
    )
    .bind(order_id)
    .fetch_one(&common_db::db().await)
    .await
    .expect("query outbox count for order_id");

    assert_eq!(count, 1, "expected exactly one outbox event for this order");
}

#[tokio::test]
#[ignore] // run via: make test-db
async fn db_idempotent_create_does_not_enqueue_outbox_twice() {
    let app = common_db::app().await;

    let external_id = format!("ord_outbox_idem_{}", uuid::Uuid::new_v4());
    let payload = format!(
        r#"{{"external_id":"{}","items":[{{"sku":"ABC","qty":1}}]}}"#,
        external_id
    );

    let key = format!("idem_outbox_{}", uuid::Uuid::new_v4());

    // First request (creates order + enqueues outbox)
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
    let order_id = body1["id"].as_str().expect("response must include id");

    // Replay (should NOT enqueue a second outbox row)
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
    assert_eq!(
        body1, body2,
        "idempotent replay should return identical response"
    );

    // Assert: still exactly one outbox row for THIS order_id
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM outbox
        WHERE event_type = 'order.created'
          AND payload->>'order_id' = $1
        "#,
    )
    .bind(order_id)
    .fetch_one(&common_db::db().await)
    .await
    .expect("query outbox count for order_id");

    assert_eq!(
        count, 1,
        "idempotent replay must not enqueue a second event"
    );
}
