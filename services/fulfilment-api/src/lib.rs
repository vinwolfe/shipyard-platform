//! Service library entrypoint.
//!
//! Why this exists:
//! - `main.rs` should be thin (runtime boot only).
//! - Tests can build the app without starting a real server.
//! - Cross-cutting concerns will be applied here over time.
//!
//! TODO: Attach global middleware/layers (request_id, tracing, metrics) at this seam.
//! TODO: Wire app state here (config, DB pool) once introduced.

use axum::Router;

/// Build the Axum application router.
///
/// Why this is a function:
/// - Stable seam for tests (`tests/*`) and the binary (`main.rs`).
/// - Central place to compose middleware/layers as the service grows.
///
/// TODO: Apply request_id and consistent error mapping globally.
/// TODO: Apply tracing + metrics layers once observability is introduced.
pub fn build_app() -> Router {
    crate::http::router::build_router()
}

pub mod http;
