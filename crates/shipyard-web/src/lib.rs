//! shipyard-web
//!
//! A small HTTP contract pack for Shipyard services.
//!
//! Provides:
//! - Request correlation (`x-request-id`) via middleware
//! - A consistent JSON error envelope (`ApiError`)
//! - A golden-path helper to apply the standard web contract to a router
//!
//! Non-goals:
//! - Tracing/metrics export (belongs in shipyard-observability)
//! - Auth, sessions, validation frameworks

pub mod contract;
pub mod error;
pub mod middleware;
pub mod request_log;

pub use contract::{apply_web_contract, not_found};
pub use error::{ApiError, ErrorBody, ErrorEnvelope};
pub use middleware::{RequestId, request_id_middleware};
pub use request_log::request_log_middleware;
