use axum::{routing::get, Router, Json};

#[tokio::main]
async fn main() {
    // println!("Hello, API!");
    let app = Router::new().route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Json<&'static str>{
    Json("hello world")
}
