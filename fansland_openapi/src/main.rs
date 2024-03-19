use axum::{
    http::{StatusCode, Uri},
    routing::get,
    Json, Router,
};
use fansland_common::RespVO;
use std::net::SocketAddr;
use tracing::{warn, Level};

use dotenv::dotenv;

use crate::handler::get_nft_ticket_qrcode;

mod api;
mod extract;
mod handler;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // build our application with some routes
    let need_auth_routers = Router::new().route("/getQRCode", get(get_nft_ticket_qrcode));

    let app_routers = need_auth_routers.fallback(fallback);

    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 3034));
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
