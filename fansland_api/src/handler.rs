use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{Json, Response},
};
use fansland_common::RespVO;
use fansland_sign::verify_signature;

use diesel::prelude::*;
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

pub async fn bind_email(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    JsonReq(req): JsonReq<BindEmailReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = pool.get().await.map_err(new_internal_error)?;
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
    State(pool): State<deadpool_diesel::postgres::Pool>,
    JsonReq(req): JsonReq<QueryAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = pool.get().await.map_err(new_internal_error)?;
    let msg_template = format!("https://fansland.io wants you to sign in with your Ethereum account:\n{}\n\nWelcome to Fansland! This request will NOT trigger a blockchain transaction or cost any gas fees.\n\nURI: https://fansland.io\nVersion: 1\nChain ID: {}\nNonce: {}\nIssued At: {}",
        req.address,
        56, // chainId
        "test-nonce", //TODO: nonce
        "test-timestamp" //TODO: timestamp,
    );
    let rsp = GetLoginNonceResp {
        address: req.address.clone(),
        signmsg: msg_template.clone(),
    };

    // TODO: 查询数据库, 判断地址是否存在，如果存在则更新，否则插入数据
    // TODO: 使用redis

    // 更新数据库
    let res = conn
        .interact(move |conn| {
            use crate::schema::users::dsl::*;
            diesel::update(users)
                .filter(user_address.eq(req.address))
                .set((
                    nonce.eq(msg_template.clone()),
                    token.eq("xxxxxxxxxxxxxxxxxxxx"), // TODO: token
                ))
                .execute(conn)
        })
        .await
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;

    tracing::debug!("res: {}", res);

    Ok(RespVO::from(&rsp).resp_json())
}

// 钱包登录
pub async fn sign_in_with_ethereum(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    JsonReq(login_req): JsonReq<LoginByAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let lrq = login_req.clone();

    let conn = pool.get().await.map_err(new_internal_error)?;
    // let uid = query_user_id;
    let usrs: Vec<User> = conn
        .interact(move |conn| {
            use crate::schema::users::dsl::*;
            users.filter(user_address.eq(login_req.address)).load(conn)
        })
        .await
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;

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
                    // TODO: 生成token, 并插入数据库(todo: redis)

                    return Ok(RespVO::from(&LoginByAddressResp {
                        success: true,
                        token: "ok-token-success".to_string(),
                    })
                    .resp_json());
                } else {
                    tracing::error!("verify sig failed");
                }
            } else {
                tracing::error!("nonce not match");
            }
        } else {
            tracing::error!("address not match");
        }
    } else {
        tracing::error!("user is empty");
    }
    new_api_error("invalid signature".to_owned())
}

pub async fn query_user_by_address(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    JsonReq(req): JsonReq<QueryAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = pool.get().await.map_err(new_internal_error)?;
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
    State(pool): State<deadpool_diesel::postgres::Pool>,
    JsonReq(req): JsonReq<QueryAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = pool.get().await.map_err(new_internal_error)?;
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
    State(pool): State<deadpool_diesel::postgres::Pool>,
    JsonReq(secret_token_req): JsonReq<GetTicketsBySecretToken>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = pool.get().await.map_err(new_internal_error)?;
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
    State(pool): State<deadpool_diesel::postgres::Pool>,
    JsonReq(update_req): JsonReq<UpdateSecretLinkPasswdReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let conn = pool.get().await.map_err(new_internal_error)?;
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
