//! Service composition seam.
//!
//! `main.rs` should be thin (runtime boot only). This module provides a stable
//! `build_app` entrypoint for both the binary and integration tests.

use axum::Router;
use shipyard_config::AppConfig;

pub mod http;
pub mod idempotency;
pub mod metrics;
pub mod outbox;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: sqlx::PgPool,
}

/// Build the Axum application router with attached state.
///
/// Runtime contract: DB is required.
/// - If you need a DB-free app for fast tests, use `build_app_without_db`.
pub fn build_app(config: AppConfig, db: sqlx::PgPool) -> Router {
    http::router::build_router().with_state(AppState { config, db })
}

/// Build an app for fast tests that do not touch the DB.
///
/// Why this exists:
/// - Keeps `cargo test` fast and DB-free by default.
/// - DB integration tests should use the real `build_app` + a real Postgres.
///
/// NOTE: Any route that requires DB should not be present in this router, and
/// `/readyz` should return 503 (not ready).
pub fn build_app_without_db(config: AppConfig) -> Router {
    http::router::build_router_no_db().with_state(config)
}
