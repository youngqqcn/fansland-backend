use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tracing::Level;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use dotenv::dotenv;

use crate::handler::{bind_email, list_tickets, list_tickets_by_userid, query_user_by_address};

pub mod handler;
pub mod model;
pub mod schema;
pub mod api;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
    //     )
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap();

    // set up connection pool
    let manager = deadpool_diesel::postgres::Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::postgres::Pool::builder(manager)
        .build()
        .unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/address/:address", get(query_user_by_address))
        .route("/address/bind", post(bind_email))
        .route("/tickets", get(list_tickets))
        .route("/tickets/:uid", get(list_tickets_by_userid))
        .with_state(pool);

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
