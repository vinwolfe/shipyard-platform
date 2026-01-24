//! API v1 router.
use axum::Router;
use shipyard_config::AppConfig;

use crate::AppState;

mod orders;

pub fn router() -> Router<AppState> {
    Router::new().nest("/orders", orders::router())
}

/// DB-free v1 router used in fast tests.
pub fn router_no_db() -> Router<AppConfig> {
    Router::new().nest("/orders", orders::router_no_db())
}
