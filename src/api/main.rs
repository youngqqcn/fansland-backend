use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // println!("Hello, API!");
    let app = Router::new()
        .route("/", get(handler))
        .route("/users", post(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Json<&'static str> {
    Json("hello world")
}

async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    tracing::debug!("create_user payload: {}", payload);
    let user = User {
        id: 11,
        username: payload.username,
    };

    (StatusCode::CREATED, Json(user)).into_response()
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

impl Display for CreateUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "username:{} ", self.username)
    }
}

#[derive(Serialize)]
struct User {
    id: i64,
    username: String,
}
