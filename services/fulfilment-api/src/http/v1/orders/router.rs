use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;
use shipyard_config::AppConfig;

use super::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/validate", post(handlers::validate::validate_order))
        .route("/", post(handlers::create::create_order))
        .route("/:id", get(handlers::get::get_order))
}

pub fn router_no_db() -> Router<AppConfig> {
    Router::new().route("/validate", post(handlers::validate::validate_order))
}
