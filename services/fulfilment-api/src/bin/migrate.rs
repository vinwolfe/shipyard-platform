//! Migration runner for fulfilment-api.
//!
//! Why a binary instead of sqlx-cli?
//! - Keeps the golden path Rust-only (no extra tooling install).
//! - Works in CI and local environments consistently.
//! - Keeps migrations service-owned and discoverable.

use std::env;

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set to run migrations");

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    // Uses the folder relative to this crate.
    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("migrations applied successfully");
    Ok(())
}
