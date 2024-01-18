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
        LoginByAddressReq, LoginByAddressResp, QueryAddressReq, QueryAddressResp,
        QueryAddressTickets, UpdateSecretLinkPasswdReq, UpdateSecretLinkPasswdResp,
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
    let prefix_key = "siwemsg:".to_string() + &req.address;
    let _: () = redis::pipe()
        .set(&prefix_key, msg_template.clone())
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
    let siwemsg_prefix_key = "siwemsg:".to_string() + &lrq.address;
    let msg = match redis::cmd("GET")
        .arg(&siwemsg_prefix_key)
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

    //  生成token
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
    let authtoken_prefix_key = "authtoken:".to_string() + &lrq.address;

    // 删除之前的msg,然后将token插入redis, 并设置过期时间为1天
    let _: () = redis::pipe()
        .del(&siwemsg_prefix_key)
        .set(&authtoken_prefix_key, &token)
        .expire(&authtoken_prefix_key, 24 * 60 * 60)
        .ignore()
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;
    tracing::debug!("插入redis成功");

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
    let req_copy = req.clone();

    let conn = app_state
        .psql_pool
        .get()
        .await
        .map_err(new_internal_error)?;
    let res: Vec<User> = conn
        .interact(move |conn| {
            use crate::schema::users::dsl::*;
            users
                .filter(user_address.eq(&req.address.clone()))
                .load(conn)
        })
        .await
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;

    let r = match res.get(0) {
        Some(u) => QueryAddressResp {
            address: u.user_address.clone(),
            email: u.email.clone(),
        },
        None => QueryAddressResp {
            address: req_copy.address,
            email: "".to_string(),
        },
    };

    Ok(RespVO::from(&r).resp_json())
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

    // 获取该用户所有的票
    let ret_tickets: Vec<Ticket> = conn
        .interact(move |conn| {
            use crate::schema::tickets::dsl::*;
            tickets.filter(user_address.eq(req.address)).load(conn)
        })
        .await
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;

    let mut r: Vec<QueryAddressTickets> = Vec::new();
    for t in &ret_tickets {
        let qrcode = match t.qrcode.clone() {
            Some(c) => c,
            None => "".to_owned(),
        };
        r.push(QueryAddressTickets {
            user_address: t.user_address.clone(),
            chain_name: t.chain_name.clone(),
            contract_address: t.contract_address.clone(),
            nft_token_id: t.nft_token_id.clone(),
            qrcode: qrcode,
            redeem_status: t.redeem_status.clone(),
            ticket_type_id: t.ticket_type_id,
            ticket_type_name: t.ticket_type_name.clone(),
            ticket_price: t.ticket_price,
            event_name: t.event_name.clone(),
            event_time: t.event_time.clone(),
        });
    }

    Ok(RespVO::from(&r).resp_json())
}

// list tickets by secret link
pub async fn get_tickets_by_secret_link(
    State(app_state): State<AppState>,
    JsonReq(secret_token_req): JsonReq<GetTicketsBySecretToken>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    // 组装key
    let req = secret_token_req.clone();
    let token_key = "slink:token:".to_string() + &req.token + &req.passwd;
    // let address_key = "slink:address:".to_string() + &req.address;

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    // 查询redis中的key
    let req_address = match redis::cmd("GET")
        .arg(&token_key)
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
        .map_err(new_internal_error)?
    {
        Some(k) => k,
        None => {
            return new_api_error("invalid secret token".to_string());
        }
    };

    let conn = app_state
        .psql_pool
        .get()
        .await
        .map_err(new_internal_error)?;

    let ret: Vec<Ticket> = conn
        .interact(move |conn| {
            use crate::schema::tickets::dsl::*;
            tickets.filter(user_address.eq(req_address)).load(conn)
        })
        .await
        .map_err(new_internal_error)?
        .map_err(new_internal_error)?;

    Ok(RespVO::from(&ret).resp_json())
}

// 更新密码
pub async fn update_secret_link_passwd(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<UpdateSecretLinkPasswdReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    tracing::debug!("===========从redis获取msg== ",);

    // 生成随机key
    let raw_token: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    let token_key = "slink:token:".to_string() + &raw_token + &req.passwd;
    let address_key = "slink:address:".to_string() + &req.address;

    // 删除旧的
    match redis::cmd("GET")
        .arg(&address_key)
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
        .map_err(new_internal_error)?
    {
        Some(k) => {
            tracing::debug!("========删除旧的slink token ======={}", &k);
            let _: () = redis::pipe()
                .del(&k)
                .ignore()
                .query_async(&mut rds_conn)
                .await
                .map_err(new_internal_error)?;
        }
        None => (),
    }

    // 设置新的
    let _: () = redis::pipe()
        .set(&token_key, &req.address)
        .persist(&token_key) // 永不过期
        .set(&address_key, &token_key)
        .persist(&address_key) // 永不过期
        .ignore()
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;

    tracing::debug!("====插入redis成功====");

    // let new_token = "new token".to_string();
    Ok(RespVO::from(&UpdateSecretLinkPasswdResp {
        success: true,
        secret_token: raw_token,
    })
    .resp_json())
}

// middleware that shows how to consume the request body upfront
pub async fn verify_token(
    headers: HeaderMap,
    app_state: &AppState,
    address: String,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
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
    // for (name, value) in hs.iter() {
    //     tracing::debug!("====== {}: {:?}", name, value);
    // }

    if !hs.contains_key("FanslandAuthToken") {
        tracing::error!("====缺少请求头  111==========");
        return new_api_error("miss header".to_string());
    }

    let value = match hs.get("FanslandAuthToken") {
        Some(k) => k,
        None => {
            tracing::error!("====缺少请求头 222==========");
            return new_api_error("miss header".to_string());
        }
    };
    let header_token = value.to_str().map_err(new_internal_error)?;
    tracing::debug!("token = {}", header_token);

    // 查询redis是否存在
    let prefix_key = "authtoken:".to_string() + &address;
    let redis_token = redis::cmd("GET")
        .arg(&prefix_key)
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;
    let token = match redis_token {
        Some(rtk) => {
            if !rtk.eq(header_token) {
                tracing::error!("========111 token 与地址不匹配==========");
                tracing::error!("========redis的token:{}", rtk);
                tracing::error!("========head的token:{}", header_token);
                return new_api_error("illegal request".to_string());
            }
            rtk
        }
        None => {
            return new_api_error("token expired, please refrese and try again".to_string());
        }
    };

    // 判断地址是否匹配
    let jt = JWTToken::verify(TOKEN_SECRET, &token).map_err(new_internal_error)?;
    if !jt.user_address().to_lowercase().eq(&address.to_lowercase()) {
        tracing::error!("========222 token 与地址不匹配==========");
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
