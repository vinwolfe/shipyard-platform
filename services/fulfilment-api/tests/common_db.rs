use axum::{Router, body::Body, http::Request, response::Response};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;

#[allow(dead_code)]
pub async fn db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for db tests (run via make test-db)");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    // Ensure schema exists for tests (safe & repeatable)
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run migrations");

    db
}

pub async fn app() -> Router {
    // Config (dev) + DB URL (tests must provide this; see make test-db)
    let config = shipyard_config::AppConfig::dev();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for db tests (run via make test-db)");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    // Ensure schema exists for tests (safe & repeatable)
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run migrations");

    fulfilment_api::build_app(config, db)
}

#[allow(dead_code)]
pub async fn send(app: Router, method: &str, uri: &str) -> Response {
    app.oneshot(
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap()
}

pub async fn send_json(app: Router, method: &str, uri: &str, body: &str) -> Response {
    app.oneshot(
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

#[allow(dead_code)]
pub async fn send_json_with_headers(
    app: Router,
    method: &str,
    uri: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");

    for (k, v) in headers {
        builder = builder.header(*k, *v);
    }

    app.oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

pub async fn body_json(res: Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}
