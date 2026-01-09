#[tokio::main]
async fn main() {
    let app = fulfilment_api::build_app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to 0.0.0.0:8080");

    println!("fulfilment-api listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await.expect("server error");
}
