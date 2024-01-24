use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use chrono::Utc;
use fansland_common::{jwt::JWTToken, RespVO};

use crate::{
    api::{
        BindEmailReq, BindEmailResp, GetLoginNonceReq, GetLoginNonceResp,
        GetTicketQrCodeBySecretToken, LoginByAddressReq, LoginByAddressResp, QueryAddressReq,
        QueryAddressResp, QueryTicketQrCodeReq, QueryTicketQrCodeResp, UpdateSecretLinkPasswdReq,
        UpdateSecretLinkPasswdResp,
    },
    extract::JsonReq,
};
use ethers::types::{Address, Signature};
use rand::Rng;
use redis_pool::{connection::RedisPoolConnection, RedisPool};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use redis::{aio::Connection, Client};

const TOKEN_SECRET: &str = "GXFC@Fansland.io@2024";

#[derive(Clone)]
pub struct AppState {
    // pub psql_pool: Pool<Manager<PgConnection>>,
    pub rds_pool: RedisPool<Client, Connection>,
}

pub async fn bind_email(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<BindEmailReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    // 设置该地址的签名msg, 验证签名的时候，直接从redis中获取即可
    // 消息设置1个小时过期时间
    let prefix_key = "bindemail:".to_string() + &req.address;
    let _: () = redis::pipe()
        .set(&prefix_key, req.email.clone())
        .persist(&prefix_key) // 持久
        .ignore()
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;

    Ok(RespVO::from(&BindEmailResp {
        success: true,
        address: req.address,
        email: req.email,
    })
    .resp_json())
}

// get login nonce
// pub async fn get_login_nonce(
pub async fn get_login_signmsg(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<GetLoginNonceReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_sig(headers.clone(), req.address.clone()).await?;
    tracing::debug!("========获取签名消息===");

    let msg_domain = std::env::var("FANSLAND_WEBSITE_URL").unwrap();
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
        .expire(prefix_key, 10 * 60)
        .ignore()
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;

    Ok(RespVO::from(&rsp).resp_json())
}

// 钱包登录
pub async fn sign_in_with_ethereum(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(login_req): JsonReq<LoginByAddressReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_sig(headers.clone(), login_req.address.clone()).await?;

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
        None => {
            // return new_api_error("not found msg, please get new sign msg again".to_string())},
            return Err((
                StatusCode::BAD_REQUEST,
                Json(RespVO::<String> {
                    code: Some(10003),
                    msg: Some("signature msg is expired".to_owned()),
                    data: None,
                }),
            ));
        }
    };

    // 比对msg是否相同
    tracing::debug!("===========msg == {}", msg);
    if !msg.eq(&login_req.msg) {
        tracing::warn!("msg is not match");
        tracing::debug!("===========msg == {}", msg);
        tracing::debug!("===========login_req.msg == {}", login_req.msg);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10002),
                msg: Some("invalid signature".to_owned()),
                data: None,
            }),
        ));
    }
    tracing::debug!("===========msg比对成功 ");

    // 验证签名
    tracing::debug!("sig: {}", lrq.sig);
    let signature = Signature::from_str(&lrq.sig.clone()).map_err(new_internal_error)?;
    let address = Address::from_str(&lrq.address.clone()).map_err(new_internal_error)?;
    signature.verify(msg.clone(), address).map_err(|e| {
        tracing::error!("===验证签名错误:{}", e.to_string());
        (
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10002),
                msg: Some("invalid signature".to_owned()),
                data: None,
            }),
        )
    })?; // TODO:

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

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    tracing::debug!("=从redis获取绑定邮箱== ",);

    // 设置该地址的签名msg, 验证签名的时候，直接从redis中获取即可
    let prefix_key = "bindemail:".to_string() + &req.address;
    let user_email = match redis::cmd("GET")
        .arg(&prefix_key)
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
        .map_err(new_internal_error)?
    {
        Some(m) => m,
        None => "".to_owned(),
    };

    Ok(RespVO::from(&QueryAddressResp {
        address: req.address,
        email: user_email,
    })
    .resp_json())
}

// list tickets
pub async fn query_ticket_qrcode_by_address(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryTicketQrCodeReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

    // 使用redis
    let rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    tracing::debug!("=从redis获取绑定tokenid对应的二维码== ",);

    query_ticket_qrcode_by_token_id(rds_conn, req.address.clone(), req.token_id).await
}

pub async fn query_ticket_qrcode_by_token_id(
    mut rds_conn: RedisPoolConnection<Connection>,
    address: String,
    token_id: u32,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    tracing::debug!("=从redis获取绑定tokenid对应的二维码== ",);

    // 从redis中获取该token_id的owner
    let prefix_key = "nft:tokenid:owner:".to_string() + &token_id.to_string();
    let token_id_owner = match redis::cmd("GET")
        .arg(&prefix_key)
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
        .map_err(new_internal_error)?
    {
        Some(m) => m,
        None => "".to_owned(),
    };

    // 比对redis中的owner与参数中的owner是否相同
    if !token_id_owner.eq(&address) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(11001),
                msg: Some("The NFT ticket is pending. Please wait a few minutes.".to_owned()),
                data: None,
            }),
        ));
    }

    // 根据算法生成二维码
    let fansland_nft_contract_address = std::env::var("FANSLAND_NFT").unwrap();
    let salt = "QrCode@fansland.io2024-888"; // TODO:
    let hash_msg = String::new()
        + &fansland_nft_contract_address
        + &token_id.to_string()
        + &token_id_owner
        + salt;
    let keccak_hash = ethers::utils::keccak256(hash_msg.as_bytes());
    let bz_qrcode = &keccak_hash[keccak_hash.len() - 15..];
    let qrcode = String::from("1:") + &hex::encode(bz_qrcode);

    let r = QueryTicketQrCodeResp {
        user_address: address,
        nft_token_id: token_id,
        qrcode: qrcode,
        // redeem_status: 0,
        // type_id: 0,
    };

    Ok(RespVO::from(&r).resp_json())
}

// 私密链接查询门票二维码
pub async fn get_ticket_qrcode_by_secret_link(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(secret_token_req): JsonReq<GetTicketQrCodeBySecretToken>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_sig(headers.clone(), secret_token_req.address.clone()).await?;

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
            return Err((
                StatusCode::BAD_REQUEST,
                Json(RespVO::<String> {
                    code: Some(10011),
                    msg: Some("link is expired".to_owned()),
                    data: None,
                }),
            ));
        }
    };

    if !req_address.eq(&req.address) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10012),
                msg: Some("bad link".to_owned()),
                data: None,
            }),
        ));
    }

    // 仅验证密码
    if req.token_id == 0xFFFFFFFF_u32 {
        return Ok((
            StatusCode::ACCEPTED,
            Json(RespVO::<String> {
                code: Some(0),
                msg: None,
                data: None,
            }),
        )
            .into_response());
    }

    // 查询门票二维码
    query_ticket_qrcode_by_token_id(rds_conn, req.address.clone(), req.token_id).await
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

    // 生成随机token
    let raw_token: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    tracing::debug!("raw_token: {}", raw_token);
    // 组装token
    let mut bz_token: Vec<u8> = Vec::new();
    bz_token.extend_from_slice(raw_token.as_bytes()); // 追加 raw_token
    bz_token.extend(
        hex::decode(&req.address.to_lowercase().replace("0x", "").to_owned())
            .map_err(new_internal_error)?,
    ); // 追加地址
    let b64_token = base64_url::encode(&bz_token);

    let token_key = "slink:token:".to_string() + &b64_token + &req.passwd;
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
        secret_token: b64_token,
    })
    .resp_json())
}

// middleware that shows how to consume the request body upfront
pub async fn verify_token(
    headers: HeaderMap,
    app_state: &AppState,
    address: String,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    // 验证签名
    let _ = verify_sig(headers.clone(), address.clone()).await?;

    tracing::debug!("==========需要鉴权接口===========");

    let hs = headers;

    // let timestamp: String  = match hs.get("Fansland-Timestamp") {
    //     Some(t) => t.to_str().map_err(new_internal_error)?
    //     None => 0,
    // }
    if !hs.contains_key("Fansland-Token") {
        tracing::error!("====缺少请求头  111==========");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(RespVO::<String> {
                code: Some(10001),
                msg: Some("unauthorized".to_owned()),
                data: None,
            }),
        ));
    }

    let value = match hs.get("Fansland-Token") {
        Some(k) => k,
        None => {
            tracing::error!("====缺少请求头 222==========");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(RespVO::<String> {
                    code: Some(10001),
                    msg: Some("unauthorized".to_owned()),
                    data: None,
                }),
            ));
        }
    };
    let header_token = value.to_str().map_err(new_internal_error)?;
    tracing::debug!("token = {}", header_token);

    // 查询redis是否存在
    // 对token进行鉴权
    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    tracing::debug!("===========从redis获取msg== ");

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
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(RespVO::<String> {
                        code: Some(10009),
                        msg: Some("illegal request".to_owned()),
                        data: None,
                    }),
                ));
            }
            rtk
        }
        None => {
            // token不存在，token过期
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(RespVO::<String> {
                    code: Some(10099),
                    msg: Some("token expired".to_owned()),
                    data: None,
                }),
            ));
        }
    };

    // 判断地址是否匹配
    let jt = JWTToken::verify(TOKEN_SECRET, &token).map_err(new_internal_error)?;
    if !jt.user_address().to_lowercase().eq(&address.to_lowercase()) {
        tracing::error!("========222 token 与地址不匹配==========");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10009),
                msg: Some("illegal request".to_owned()),
                data: None,
            }),
        ));
    }

    tracing::debug!("=========token与地址匹配=========");

    // let rsp = next.run(request).await;
    Ok(().into_response())
}

pub async fn verify_sig(
    headers: HeaderMap,
    address: String,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    tracing::debug!("==========需要验证签名接口===========");

    let hs = headers;
    // for (name, value) in hs.iter() {
    //     tracing::debug!("====== {}: {:?}", name, value);
    // }
    if !hs.contains_key("Fansland-Timestamp") || !hs.contains_key("Fansland-Nonce") {
        tracing::error!("====缺少请求头  111==========");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10004),
                msg: Some("invalid sig".to_owned()),
                data: None,
            }),
        ));
    }

    let mut api_timestamp = 0_u128;
    let mut api_nonce = 0_u64;
    if let Some(ts) = hs.get("Fansland-Timestamp") {
        if let Ok(ts) = ts.to_str() {
            if let Ok(r) = ts.parse::<u128>() {
                api_timestamp = r;
            }
        }
    }

    if let Some(nonce_value) = hs.get("Fansland-Nonce") {
        if let Ok(nonce_str) = nonce_value.to_str() {
            if let Ok(nonce) = nonce_str.parse::<u64>() {
                api_nonce = nonce;
            }
        }
    }

    if api_timestamp < 1705977612000 || api_nonce < 10000 {
        tracing::error!("====签名字段无效==========");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10004),
                msg: Some("invalid sig".to_owned()),
                data: None,
            }),
        ));
    }

    // 接口签名验签
    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(new_internal_error)?
        .as_millis();
    // 正负5秒， 即时间窗口有10s
    if now_ts.abs_diff(api_timestamp) > 5_000 {
        tracing::error!("====时间戳无效==========");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10004),
                msg: Some("invalid sig".to_owned()),
                data: None,
            }),
        ));
    }

    let sig_msg = format!("fansland_{address}_{api_timestamp}_{api_nonce}");
    tracing::info!(sig_msg);
    let bz_sig = blake3::hash(sig_msg.as_bytes()).as_bytes().clone();
    if bz_sig[0] != 0 {
        tracing::error!("====bz[0]不为0,签名无效==========");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10004),
                msg: Some("invalid sig".to_owned()),
                data: None,
            }),
        ));
    }

    // let rsp = next.run(request).await;
    Ok(().into_response())
}

// 10000: 服务器内部错误
// 10001: unauthorized 未鉴权
// 10002: invalid signature 签名消息不合法
// 10002: signature msg is illegal 签名消息不合法
// 10003: signature msg is expired 签名消息过期
// 10004: sig invalid 接口签名无效
// 10005: invalid timestamp 时间戳无效
// 10009: illegal request 非法请求
// 10011: link is expired 私密链接过期
// 10012: bad link 私密链接请求非法
// 11001: nft ticket is pending, please wait few minutes 门票二维码还在生成中，请稍等几分钟
// 10099: token expired token过期

fn new_internal_error<E>(err: E) -> (StatusCode, Json<RespVO<String>>)
where
    E: std::error::Error,
{
    let msg = format!("server error: {}", err.to_string());
    tracing::error!("ERROR: {}", msg.clone());
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(RespVO::<String> {
            code: Some(10000),
            msg: Some("Sorry, please try again.".to_string()),
            data: None,
        }),
    )
}

// fn new_api_error(err: String) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
//     let msg = format!("server error: : {}", err.to_string());
//     warn!("{}", msg.clone());
//     Err((
//         StatusCode::INTERNAL_SERVER_ERROR,
//         Json(RespVO::<String> {
//             code: Some(10000),
//             msg: Some(msg),
//             data: None,
//         }),
//     ))
// }
