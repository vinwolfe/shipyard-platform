//! API v1 router.

use axum::Router;

use crate::AppState;

mod orders;

pub fn router() -> Router<AppState> {
    Router::new().nest("/orders", orders::router())
}
