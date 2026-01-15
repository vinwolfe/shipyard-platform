//! HTTP middleware layer.
//!
//! Why this module exists:
//! - Groups cross-cutting HTTP behaviours (request_id, later tracing/metrics).
//!
//! TODO: Add additional middleware (timeouts, tracing, metrics) as they become concrete needs.

pub mod request_id;

pub use request_id::RequestId;
