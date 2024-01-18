use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use chrono::Utc;
use fansland_common::{error::Error, jwt::JWTToken, RespVO};
// use fansland_sign::verify_signature;

use diesel::prelude::*;
use redis_pool::RedisPool;
use tracing::warn;

use crate::{
    api::{
        BindEmailReq, BindEmailResp, GetLoginNonceReq, GetLoginNonceResp, GetTicketsBySecretToken,
        LoginByAddressReq, LoginByAddressResp, QueryAddressReq, UpdateSecretLinkPasswdReq,
        UpdateSecretLinkPasswdResp,
    },
    extract::JsonReq,
    model::*,
    schema::users::{self},
};
use ethers::types::{Address, Signature};
use rand::Rng;
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use deadpool_diesel::{Manager, Pool};
use diesel::PgConnection;
// use fansland_common::RespVO;
use redis::{aio::Connection, Client};

const TOKEN_SECRET: &str = "GXFC@Fansland.io@2024";

#[derive(Clone)]
pub struct AppState {
    pub psql_pool: Pool<Manager<PgConnection>>,
    pub rds_pool: RedisPool<Client, Connection>,
}

pub async fn bind_email(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<BindEmailReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

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
    JsonReq(req): JsonReq<GetLoginNonceReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let msg_domain = "localhost:8000";
    let msg_nonce = rand::thread_rng().gen_range(10_000_000..=99_999_999); // 必须是8位数整数
    let msg_timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let msg_template = format!("{} wants you to sign in with your Ethereum account:\n{}\n\nWelcome to Fansland!\n\nURI: {}\nVersion: 1\nChain ID: {}\nNonce: {}\nIssued At: {}",
        msg_domain,
        req.address.clone(),
        msg_domain,
        req.chainid,
        msg_nonce,
        msg_timestamp
    );

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
        .create_token(TOKEN_SECRET)
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
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

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
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

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
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<UpdateSecretLinkPasswdReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

    let conn = app_state
        .psql_pool
        .get()
        .await
        .map_err(new_internal_error)?;
    let req = req.clone();

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

// middleware that shows how to consume the request body upfront
pub async fn verify_token(
    headers: HeaderMap,
    app_state: &AppState,
    address: String,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    // ) -> Response {
    // let request = buffer_request_body(request).await?;
    tracing::debug!("==========需要鉴权接口===========");

    // 对token进行鉴权
    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    tracing::debug!("===========从redis获取msg== ");

    let hs = headers;
    for (name, value) in hs.iter() {
        tracing::debug!("====== {}: {:?}", name, value);
    }

    if !hs.contains_key("FanslandAuthToken") {
        return new_api_error("miss header".to_string());
    }

    let value = match hs.get("FanslandAuthToken") {
        Some(k) => k,
        None => {
            return new_api_error("miss header".to_string());
        }
    };
    let token = value.to_str().unwrap(); // TODO: fix
    tracing::debug!("token = {}", token);

    // 查询redis是否存在
    let rk = redis::cmd("GET")
        .arg(token)
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
        .unwrap();
    match rk {
        Some(_) => {}
        None => {
            return new_api_error("token expired, please refrese and try again".to_string());
        }
    }

    // 判断地址是否匹配
    let jt = JWTToken::verify(TOKEN_SECRET, token).map_err(new_internal_error)?;
    if !jt.user_address().to_lowercase().eq(&address.to_lowercase()) {
        tracing::error!("========token 与地址不匹配==========");
        return new_api_error("illegal requst".to_string());
    }

    tracing::debug!("=========token与地址匹配=========");

    // let rsp = next.run(request).await;
    Ok(().into_response())
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

// fn map_api_error<E>(err: E) -> Error
// where
//     E: std::error::Error,
// {
//     Error::E(err.to_string())
// }

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
