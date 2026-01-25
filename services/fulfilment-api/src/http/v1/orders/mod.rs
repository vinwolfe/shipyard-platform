//! Orders routes for API v1.
//!
//! This module is split by responsibility:
//! - `handlers`: Axum handler glue (extractors, status codes, response shapes)
//! - `repo`: SQLx queries + DB error mapping
//! - `validate`: request validation helpers

mod router;
pub use router::{router, router_no_db};

mod handlers;
mod repo;
mod types;
mod validate;
