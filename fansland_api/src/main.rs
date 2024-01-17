use axum::{
    extract::Request,
    http::{StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use fansland_common::RespVO;
use std::net::SocketAddr;
use tracing::{warn, Level};

use dotenv::dotenv;

use crate::handler::{
    bind_email, get_login_signmsg, get_tickets_by_secret_link, query_tickets_by_address,
    query_user_by_address, sign_in_with_ethereum, update_secret_link_passwd,
};

pub mod api;
mod extract;
pub mod handler;
pub mod model;
pub mod schema;

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
    let need_auth_routers = Router::new()
        .route("/queryAddress", post(query_user_by_address))
        .route("/bindEmail", post(bind_email))
        .route("/queryTicketsByAddress", post(query_tickets_by_address))
        .route("/updateSlink", post(update_secret_link_passwd))
        .layer(middleware::from_fn(print_request_body));

    let noneed_auth_routers = Router::new()
        .route("/slink", post(get_tickets_by_secret_link))
        .route("/getSiweMsg", post(get_login_signmsg))
        .route("/signInWithEthereum", post(sign_in_with_ethereum));

    let app_routers = need_auth_routers
        .merge(noneed_auth_routers)
        .fallback(fallback)
        .with_state(pool);

    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app_routers).await.unwrap();
}

// 处理链接不存在的情况
async fn fallback(uri: Uri) -> Result<String, (StatusCode, Json<RespVO<String>>)> {
    let msg = format!("NOT FOUND: {}", uri);
    warn!("{}", msg.clone());
    Err((
        StatusCode::NOT_FOUND,
        Json(RespVO::<String> {
            code: Some(-1),
            msg: Some(msg),
            data: None,
        }),
    ))
}
// async fn fallback(uri: Uri) -> impl IntoResponse {
//     let msg = format!("not found: {}", uri);
//     warn!("{}", msg.clone());
//     RespVO::<String> {
//         code: Some(-1),
//         msg: Some(msg),
//         data: None,
//     }
//     .resp_json()
// }

//====================

// middleware that shows how to consume the request body upfront
async fn print_request_body(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
    // let request = buffer_request_body(request).await?;
    tracing::debug!("==========需要鉴权接口===========");

    let hs = request.headers();
    for (name, value) in hs.iter() {
        tracing::debug!("====== {}: {:?}", name, value);
    }
    // tracing::debug!("{}", request.headers());
    Ok(next.run(request).await)
}

// pub struct AppState {
//     pub pool: Pool<Manager<PgConnection>>,
//     pub rdc: redis::Client,
// }
