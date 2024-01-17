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
    api::{
        BindEmailReq, BindEmailResp, GetLoginNonceResp, GetTicketsBySecretToken, LoginByAddressReq,
        LoginByAddressResp,
    },
    model::*,
    schema::{
        tickets::{self, user_id},
        users::{self, user_address},
    },
};

pub async fn bind_email(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(req): Json<BindEmailReq>,
) -> Result<Json<BindEmailResp>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let _ = conn
        .interact(move |conn| {
            let xuser = CreateUser {
                user_address: req.address,
                email: req.email,
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

    let res = conn
        .interact(move |conn| {
            use crate::schema::users::dsl::*;
            diesel::update(users)
                .filter(user_address.eq(addr))
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
    let lrq = login_req.clone();

    let conn = pool.get().await.map_err(internal_error)?;
    // let uid = query_user_id;
    let usrs: Vec<User> = conn
        .interact(move |conn| {
            use crate::schema::users::dsl::*;
            users.filter(user_address.eq(login_req.address)).load(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    tracing::debug!("len of usrs : {}", usrs.len());

    if let Some(usr) = usrs.get(0) {
        tracing::debug!(
            "usr.address: {}, lrq.address:{}",
            usr.user_address,
            lrq.address
        );
        if usr.user_address.eq(&lrq.address) {
            tracing::debug!("usr.nonce: {}, lrq.msg:{}", usr.nonce, lrq.msg);
            if usr.nonce.eq(&lrq.msg) {
                // 验证签名 + 消息
                if verify_signature(lrq.msg, lrq.sig, lrq.address) {
                    // TODO: 生成token

                    return Ok(Json(LoginByAddressResp {
                        success: true,
                        token: "ok-token-success".to_string(),
                    }));
                } else {
                    return Err((StatusCode::BAD_REQUEST, "verify sig failed".to_string()));
                }
            } else {
                return Err((StatusCode::BAD_REQUEST, "nonce not match".to_string()));
            }
        } else {
            return Err((StatusCode::BAD_REQUEST, "address not match".to_string()));
        }
    } else {
        Err((StatusCode::BAD_REQUEST, " user is empty".to_string()))
    }
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
            users.filter(user_address.eq(addr)).load(conn)
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
pub async fn get_tickets_by_secret_token(
    // Path(uid): Path<i64>,
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(secret_token_req): Json<GetTicketsBySecretToken>,
) -> Result<Json<Vec<Ticket>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let req: GetTicketsBySecretToken = secret_token_req.clone();

    // 查询用户
    let res: Vec<User> = conn
        .interact(|conn| {
            use crate::schema::users::dsl::*;
            users
                .filter(user_address.eq(req.address))
                .filter(token.eq(req.token))
                .filter(passwd.eq(req.passwd))
                .load(conn)

            // use crate::schema::tickets::dsl::*;
            // tickets.filter(user_id.eq(uid)).load(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    if let Some(usr) = res.get(0) {
        // 获取该用户所有的票
        let usr_address = usr.user_address.clone();
        let ret: Vec<Ticket> = conn
            .interact(move |conn| {
                use crate::schema::tickets::dsl::*;
                tickets.filter(user_address.eq(usr_address)).load(conn)
            })
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        return Ok(Json(ret));
    }

    return Err((StatusCode::BAD_REQUEST, "invalid".to_string()));
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
