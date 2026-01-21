//! HTTP layer for the service.
//!
//! Why it exists:
//! - Keeps routing/handlers separate from app assembly (`lib.rs`).
//! - Keeps the public seam (`build_app`) stable while HTTP internals evolve.
//!
//! TODO: Introduce shared HTTP utilities (error mapping, middleware) when they emerge as stable contracts.

pub mod middleware;
pub mod router;
pub mod v1;
