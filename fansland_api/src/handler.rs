use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{Json, Response},
};
use fansland_common::{jwt::JWTToken, RespVO};
// use fansland_sign::verify_signature;

use diesel::prelude::*;
use redis_pool::RedisPool;
use tracing::warn;

use crate::{
    api::{
        BindEmailReq, BindEmailResp, GetLoginNonceResp, GetTicketsBySecretToken, LoginByAddressReq,
        LoginByAddressResp, QueryAddressReq, UpdateSecretLinkPasswdReq, UpdateSecretLinkPasswdResp,
    },
    extract::JsonReq,
    model::*,
    schema::users::{self},
};
use ethers::types::{Address, Signature};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use deadpool_diesel::{Manager, Pool};
use diesel::PgConnection;
// use fansland_common::RespVO;
use redis::{aio::Connection, Client};

#[derive(Clone)]
pub struct AppState {
    pub psql_pool: Pool<Manager<PgConnection>>,
    pub rds_pool: RedisPool<Client, Connection>,
}

pub async fn bind_email(
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<BindEmailReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = app_state
        .psql_pool
        .get()
        .await
        .map_err(new_internal_error)?;
    let _ = conn
        .interact(move |conn| {
            // TODO: 判断地址是否存在？
            // TODO: 判断邮箱是否存在
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
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;
    Ok(RespVO::from(&BindEmailResp { success: true }).resp_json())
}

// get login nonce
// pub async fn get_login_nonce(
pub async fn get_login_signmsg(
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    // let conn = app_state
    //     .psql_pool
    //     .get()
    //     .await
    //     .map_err(new_internal_error)?;
    // let msg_template = format!("https://fansland.io wants you to sign in with your Ethereum account:\n{}\n\nWelcome to Fansland! This request will NOT trigger a blockchain transaction or cost any gas fees.\n\nURI: https://fansland.io\nVersion: 1\nChain ID: {}\nNonce: {}\nIssued At: {}",
    //     req.address,
    //     56, // chainId
    //     "test-nonce", //TODO: nonce
    //     "test-timestamp" //TODO: timestamp,
    // );

    let nonce = "uuid-todo";
    let timestamp = "2021-09-30T16:25:24.000Z";

    let msg_template = String::from( "localhost:9011 wants you to sign in with your Ethereum account:\n\n0xbfe5f435389ca190c3d3dec351db0ee9a8657a53\n\nI accepted the MetaMask Terms of Service: https://community.metamask.io/tos\n\nURI: https://localhost:9011\nVersion: 1\nChain ID: 1\nNonce: 32891757\nIssued At: 2021-09-30T16:25:24.000Z");

    let rsp = GetLoginNonceResp {
        address: req.address.clone(),
        signmsg: msg_template.clone(),
    };

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    // 设置该地址的签名msg, 验证签名的时候，直接从redis中获取即可
    // 消息设置1个小时过期时间
    let _: () = redis::pipe()
        .set(req.address.clone(), msg_template.clone())
        .expire(req.address.clone(), 10 * 60)
        .ignore()
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;

    Ok(RespVO::from(&rsp).resp_json())
}

// 钱包登录
pub async fn sign_in_with_ethereum(
    State(app_state): State<AppState>,
    JsonReq(login_req): JsonReq<LoginByAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let lrq = login_req.clone();

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    tracing::debug!("===========从redis获取msg== ",);

    // 设置该地址的签名msg, 验证签名的时候，直接从redis中获取即可
    let msg = match redis::cmd("GET")
        .arg(lrq.address.clone())
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
        .map_err(new_internal_error)?
    {
        Some(m) => m,
        None => return new_api_error("not found msg, please get new sign msg again".to_string()),
    };

    // 比对msg是否相同
    tracing::debug!("===========msg == {}", msg);
    if !msg.eq(&login_req.msg) {
        tracing::warn!("msg is not match");
        return new_api_error("invalid sig".to_string());
    }
    tracing::debug!("===========msg比对成功 ");

    // 验证签名
    let signature = Signature::from_str(&lrq.sig.clone()).map_err(new_internal_error)?;
    let address = Address::from_str(&lrq.address.clone()).map_err(new_internal_error)?;
    signature
        .verify(msg.clone(), address)
        .map_err(new_internal_error)?;

    tracing::debug!("============验证签名成功");

    // TODO: 生成token
    // let token = String::from("token-todo-expire");
    let mut jwt_token = JWTToken::default();
    jwt_token.set_user_address(lrq.address.clone());
    jwt_token.set_exp(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(new_internal_error)?
            .as_secs()
            + 80000,
    );
    let token = jwt_token
        .create_token("GXFC@Fansland.io@2024")
        .map_err(new_internal_error)?;

    // 删除之前的msg,然后将token插入redis, 并设置过期时间为1天
    let _: () = redis::pipe()
        .del(lrq.address.clone())
        .set(&token, lrq.address.clone())
        .expire(&token, 24 * 60 * 60)
        .ignore()
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;
    tracing::debug!("插入redis成功");

    // TODO: 插入数据库？

    // 返回鉴权token
    Ok(RespVO::from(&LoginByAddressResp {
        success: true,
        address: lrq.address.clone(),
        token: token,
    })
    .resp_json())
}

pub async fn query_user_by_address(
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = app_state
        .psql_pool
        .get()
        .await
        .map_err(new_internal_error)?;
    let res: Vec<User> = conn
        .interact(move |conn| {
            use crate::schema::users::dsl::*;
            users.filter(user_address.eq(req.address)).load(conn)
        })
        .await
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;

    Ok(RespVO::from(&res).resp_json())
}

// list tickets
pub async fn query_tickets_by_address(
    // Path(addr): Path<String>,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = app_state
        .psql_pool
        .get()
        .await
        .map_err(new_internal_error)?;
    //TODO: 在中间件中校验token的合法性

    // 获取该用户所有的票
    let ret: Vec<Ticket> = conn
        .interact(move |conn| {
            use crate::schema::tickets::dsl::*;
            tickets.filter(user_address.eq(req.address)).load(conn)
        })
        .await
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;

    Ok(RespVO::from(&ret).resp_json())
}

// list tickets by secret link
pub async fn get_tickets_by_secret_link(
    State(app_state): State<AppState>,
    JsonReq(secret_token_req): JsonReq<GetTicketsBySecretToken>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = app_state
        .psql_pool
        .get()
        .await
        .map_err(new_internal_error)?;
    let req = secret_token_req.clone();

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
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;

    if let Some(usr) = res.get(0) {
        // 获取该用户所有的票
        let usr_address = usr.user_address.clone();
        let ret: Vec<Ticket> = conn
            .interact(move |conn| {
                use crate::schema::tickets::dsl::*;
                tickets.filter(user_address.eq(usr_address)).load(conn)
            })
            .await
            .map_err(new_internal_error)?
            .map_err(new_internal_error)?;

        return Ok(RespVO::from(&ret).resp_json());
    }

    return new_api_error("invalid param".to_owned());
}

// 更新密码
pub async fn update_secret_link_passwd(
    State(app_state): State<AppState>,
    JsonReq(update_req): JsonReq<UpdateSecretLinkPasswdReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = app_state
        .psql_pool
        .get()
        .await
        .map_err(new_internal_error)?;
    let req = update_req.clone();

    // 查询用户私密链接密码
    let _res = conn
        .interact(|conn| {
            use crate::schema::users::dsl::*;
            diesel::update(users)
                .filter(user_address.eq(req.address))
                .set(passwd.eq(req.passwd))
                .execute(conn)
        })
        .await
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;

    // TODO: create new token
    let new_token = "new token".to_string();
    Ok(RespVO::from(&UpdateSecretLinkPasswdResp {
        success: true,
        secret_token: new_token,
    })
    .resp_json())
}

fn new_internal_error<E>(err: E) -> (StatusCode, Json<RespVO<String>>)
where
    E: std::error::Error,
{
    let msg = format!("INTERNAL_SERVER_ERROR: {}", err.to_string());
    warn!("{}", msg.clone());
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(RespVO::<String> {
            code: Some(-1),
            msg: Some(msg),
            data: None,
        }),
    )
}

fn new_api_error(err: String) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let msg = format!("INTERNAL_SERVER_ERROR: {}", err.to_string());
    warn!("{}", msg.clone());
    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(RespVO::<String> {
            code: Some(-1),
            msg: Some(msg),
            data: None,
        }),
    ))
}
