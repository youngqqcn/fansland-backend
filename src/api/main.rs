use axum::{routing::get, Router, Json};
use tracing::Level;

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();

    // println!("Hello, API!");
    let app = Router::new().route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Json<&'static str>{
    Json("hello world")
}
