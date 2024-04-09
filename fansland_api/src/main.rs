use axum::{
    http::{StatusCode, Uri},
    routing::post,
    Json, Router,
};
use clap::Parser;
use fansland_common::RespVO;
use redis::Client;
use redis_pool::RedisPool;
use std::net::SocketAddr;
use tracing::{warn, Level};

use dotenv::dotenv;

use crate::handler::{
    bind_email, get_login_signmsg, get_ticket_qrcode_by_secret_link,
    query_ticket_qrcode_by_address, query_user_by_address, query_whitelist, sign_in_with_ethereum,
    update_secret_link_passwd, AppState,
};

mod api;
mod extract;
mod handler;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// env ,  [test: for test env], [uat: for uat env], [pro: for pro env]
    #[arg(short, long)]
    #[arg()]
    env: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let args = Args::parse();

    let web_domain = match args.env.as_str() {
        "test" => "test-land.fansland.xyz",
        "uat" => "uat.fansland.io",
        "pro" => "fansland.io",
        _ => panic!("invalid env"),
    };

    // tracing_subscriber::registry()
    //     .with(s
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
    //     )
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let rds_url = std::env::var("REDIS_URL").unwrap();
    tracing::debug!("rds_url: {}", rds_url);
    let client = Client::open(rds_url).unwrap();
    let redis_pool = RedisPool::from(client);

    let app_state = AppState {
        rds_pool: redis_pool.clone(),
        web_domain: web_domain.to_owned(),
        env: args.env.to_uppercase(),
    };

    // build our application with some routes
    let need_auth_routers = Router::new()
        .route("/queryAddressEmail", post(query_user_by_address))
        .route("/bindEmail", post(bind_email))
        .route(
            "/queryQrCodeByAddress",
            post(query_ticket_qrcode_by_address),
        )
        .route("/updateSlink", post(update_secret_link_passwd));

    let noneed_auth_routers = Router::new()
        .route(
            "/queryQrCodeBySlink",
            post(get_ticket_qrcode_by_secret_link),
        )
        .route("/getSiweMsg", post(get_login_signmsg))
        .route("/checkWhitelist", post(query_whitelist))
        .route("/signInWithEthereum", post(sign_in_with_ethereum));

    let app_routers = need_auth_routers
        .merge(noneed_auth_routers)
        .fallback(fallback)
        .with_state(app_state.clone());

    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 3033));
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
