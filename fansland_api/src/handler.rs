use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use fansland_sign::verify_signature;

use diesel::prelude::*;
// use std::net::SocketAddr;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    api::{BindEmailReq, BindEmailResp, GetLoginNonceResp, LoginByAddressReq, LoginByAddressResp},
    model::*,
    schema::{
        tickets::{self, user_id},
        users::{self},
    },
};

pub async fn bind_email(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(new_user): Json<BindEmailReq>,
) -> Result<Json<BindEmailResp>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let _ = conn
        .interact(move |conn| {
            let xuser = CreateUser {
                address: new_user.address,
                email: new_user.email,
                nonce: "noce".to_string(),
                token: "token".to_string(),
            };

            diesel::insert_into(users::table)
                .values(xuser)
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(BindEmailResp { success: true }))
}

// get login nonce
// pub async fn get_login_nonce(
pub async fn get_login_signmsg(
    Path(addr): Path<String>,
    State(pool): State<deadpool_diesel::postgres::Pool>,
) -> Result<Json<GetLoginNonceResp>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let msg_template = format!("https://fansland.io wants you to sign in with your Ethereum account:\n{}\n\nWelcome to Fansland! This request will NOT trigger a blockchain transaction or cost any gas fees.\n\nURI: https://fansland.io\nVersion: 1\nChain ID: {}\nNonce: {}\nIssued At: {}",
        addr,
        56, // chainId
        "test-nonce", //nonce
        "test-timestamp" // timestamp
    );
    let rsp = GetLoginNonceResp {
        address: addr.clone(),
        signmsg: msg_template.clone(),
    };

    // let rand_nonce = "TODO";
    let res = conn
        .interact(move |conn| {
            use crate::schema::users::dsl::*;
            diesel::update(users)
                .filter(address.eq(addr))
                .set((
                    nonce.eq(msg_template.clone()),
                    token.eq("xxxxxxxxxxxxxxxxxxxx"),
                ))
                .execute(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    tracing::debug!("res: {}", res);
    Ok(Json(rsp))
}

// login_by_address
pub async fn login_by_address(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(login_req): Json<LoginByAddressReq>,
) -> Result<Json<LoginByAddressResp>, (StatusCode, String)> {
    // TODO: 验证签名 + 消息
    // 使用工具： https://arbiscan.io/verifiedSignatures#

    // verify_signature();

    Ok(Json(LoginByAddressResp {
        success: true,
        token: "XXXXXXTOEKN".to_string(),
    }))
}

pub async fn query_user_by_address(
    Path(addr): Path<String>,
    State(pool): State<deadpool_diesel::postgres::Pool>,
    // Json(new_user): Json<NewUser>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    // let uid = query_user_id;
    let res = conn
        .interact(move |conn| {
            use crate::schema::users::dsl::*;
            users.filter(address.eq(addr)).load(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

pub async fn list_users(
    State(pool): State<deadpool_diesel::postgres::Pool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| users::table.select(User::as_select()).load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

// list tickets
pub async fn list_tickets(
    State(pool): State<deadpool_diesel::postgres::Pool>,
) -> Result<Json<Vec<Ticket>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| tickets::table.select(Ticket::as_select()).load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

// list tickets by userid
pub async fn list_tickets_by_userid(
    Path(uid): Path<i64>,
    State(pool): State<deadpool_diesel::postgres::Pool>,
    // Json(new_user): Json<NewUser>,
) -> Result<Json<Vec<Ticket>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    // let uid = query_user_id;
    let res = conn
        .interact(move |conn| {
            use crate::schema::tickets::dsl::*;
            tickets.filter(user_id.eq(uid)).load(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
