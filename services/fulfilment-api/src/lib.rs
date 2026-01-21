//! Service composition seam.
//!
//! `main.rs` should be thin (runtime boot only). This module provides a stable
//! `build_app` entrypoint for both the binary and integration tests.

use axum::Router;
use shipyard_config::AppConfig;

pub mod http;
pub mod metrics;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
}

/// Build the Axum application router with attached state.
///
/// TODO: Attach tracing + metrics layers at this seam once observability is introduced.
pub fn build_app(config: AppConfig) -> Router {
    http::router::build_router().with_state(AppState { config })
}
