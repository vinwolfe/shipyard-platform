//! API v1 router.
//!
//! Why this module exists:
//! - Keeps versioning explicit and contained.
//! - Allows introducing v2+ later without rewriting the entire router.
//! - Encourages a predictable module layout: `v1/{resource}.rs`.
//!
//! TODO: Add additional resource routers under v1 as the API grows.

use axum::Router;

mod orders;

/// Build the v1 router.
///
/// Design notes:
/// - Nest resources under a clear namespace (`/orders`, `/customers`, etc.).
pub fn router() -> Router {
    Router::new().nest("/orders", orders::router())
}
