use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use chrono::{DateTime, Local, Utc};
use email_address::EmailAddress;
use fansland_common::{jwt::JWTToken, utils, RespVO};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::mysql::MySqlPoolOptions;
use uuid::Uuid;

use crate::{
    api::{
        AIChatConfigReq, AIChatConfigResp, AIChatHistoryReq, AIChatHistoryResp, AIChatReq,
        AIChatResp, BindEmailReq, BindEmailResp, GetLoginNonceReq, GetLoginNonceResp,
        GetTicketQrCodeBySecretTokenReq, HistoryChatMsgBody, LoginByAddressReq, LoginByAddressResp,
        MsgBody, OpenloveResp, Point, QueryAddressPointsHistoryReq, QueryAddressPointsHistoryResp,
        QueryAddressPointsReq, QueryAddressPointsResp, QueryAddressReq, QueryAddressResp,
        QueryPointsRankReq, QueryPointsRankResp, QueryTicketQrCodeReq, QueryTicketQrCodeResp,
        QueryWhitelistReq, QueryWhitelistResp, Rank, UpdateSecretLinkPasswdReq,
        UpdateSecretLinkPasswdResp,
    },
    extract::JsonReq,
};
use ethers::types::{Address, Signature};
use rand::Rng;
use redis_pool::{connection::RedisPoolConnection, RedisPool};
use std::{
    collections::HashMap,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use redis::{aio::Connection, Client};

const TOKEN_SECRET: &str = "GXFC@Fansland.io@2024";

#[derive(Clone)]
pub struct AppState {
    // pub psql_pool: Pool<Manager<PgConnection>>,
    pub rds_pool: RedisPool<Client, Connection>,
    pub web_domain: String,
    pub env: String,
    pub database_url: String,
    pub openlove_url: String,
}

// 获取积分排行榜
#[allow(dead_code)]
pub async fn get_points_rank(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryPointsRankReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_sig(headers.clone(), req.address.clone()).await?;

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    // 查询地址积分
    // 通过命令: ZREVRANGEBYSCORE points:0x51bdbad59a24207b32237e5c47e866a32a8d5ed8 9999999999 0 WITHSCORES
    // let point_prefix_key = String::from("points:") + &req.address.to_lowercase();
    // tracing::info!("{point_prefix_key}");

    let start_index: isize = (req.page * req.page_size) as isize;
    let end_index: isize = start_index + req.page_size as isize - 1;

    let rank_prefix_key: String = String::from("pointsrank");
    let points_rank_ret: Vec<Vec<String>> = redis::pipe()
        .zrevrange_withscores(&rank_prefix_key, start_index, end_index)
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;

    tracing::info!("points rank: {:?}", points_rank_ret);
    tracing::info!("points rank length: {:?}", points_rank_ret[0]);

    // 获取总条数
    let total_counts: Vec<u32> = redis::pipe()
        .zcard(&rank_prefix_key)
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;

    // let total_counts = if total_counts_ret.len() > 0 {
    //     total_counts_ret[0]
    //         .as_ref()
    //         .unwrap_or(&String::from("0"))
    //         .parse()
    //         .unwrap_or(0)
    // } else {
    //     0
    // };

    let mut ranks: Vec<Rank> = Vec::new();
    if points_rank_ret[0].len() > 0 {
        for index in 0..(&points_rank_ret[0].len() / 2) {
            let idx = index * 2;
            let rank_no = 1 + (start_index + index as isize) as u32;
            ranks.push(Rank {
                rank_no: rank_no,
                address: points_rank_ret[0][idx].clone(),
                points: points_rank_ret[0][idx + 1].parse().unwrap(),
            });
        }
    }

    Ok(RespVO::from(&QueryPointsRankResp {
        total: total_counts[0],
        page: req.page,
        page_size: req.page_size,
        rank: ranks,
    })
    .resp_json())
}

// 获取地址积分
#[allow(dead_code)]
pub async fn get_address_points(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryAddressPointsReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    // let _ = verify_sig(headers.clone(), req.address.clone()).await?;
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    //==============方案1：查询地址积分并计算 ===============
    // 通过命令: ZREVRANGEBYSCORE points:0x51bdbad59a24207b32237e5c47e866a32a8d5ed8 9999999999 0 WITHSCORES
    let point_prefix_key = String::from("points:") + &req.address.to_lowercase();
    tracing::info!("{point_prefix_key}");
    let points_ret: Vec<Vec<String>> = redis::pipe()
        .zrange(point_prefix_key, 0_isize, 5000)
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;

    tracing::info!("points history: {:?}", points_ret);

    let mut total_points = 0_u32;
    if points_ret.len() > 0 && points_ret[0].len() > 0 {
        tracing::info!("points history length: {:?}", points_ret[0].len());
        for item in &points_ret[0] {
            let parts: Vec<&str> = item.split('_').collect();
            total_points += parts[1].parse::<u32>().unwrap();
        }
    }
    tracing::info!("total points : {:?}", total_points);

    // ============= 方案2： 从积分排行榜中获取 ===============
    // let rank_prefix_key: String = String::from("pointsrank");
    // let member_key = &req.address.to_lowercase();
    // let total_ret: Vec<Option<String>> = redis::pipe()
    //     .zscore(rank_prefix_key, member_key)
    //     .query_async(&mut rds_conn)
    //     .await
    //     .map_err(new_internal_error)?;

    // tracing::info!("total points: {:?}", total_ret);
    // let total_points = if total_ret.len() > 0 {
    //     total_ret[0]
    //         .as_ref()
    //         .unwrap_or(&String::from("0"))
    //         .parse()
    //         .unwrap_or(0)
    // } else {
    //     0
    // };
    //===============================================

    Ok(RespVO::from(&QueryAddressPointsResp {
        address: req.address,
        points: total_points,
    })
    .resp_json())
}

// 获取地址是否是白名单
pub async fn query_whitelist(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryWhitelistReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_sig(headers.clone(), req.address.clone()).await?;

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    let redeemaddress_key = "whitelists:advance:0410";
    let member = format!("{}", req.address.to_lowercase());
    let is_redeem_ret: Vec<u64> = redis::pipe()
        .sismember(redeemaddress_key, member)
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;

    let is_whitelist = is_redeem_ret[0] == 1;
    if is_whitelist {
        tracing::info!("Ok, {} 是Advance-0410白名单地址", req.address);
    } else {
        tracing::info!("Sorry, {} 不是Advance-0410白名单地址", req.address);
    }

    Ok(RespVO::from(&QueryWhitelistResp {
        address: req.address,
        is_whitelist: is_whitelist,
    })
    .resp_json())
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
struct ChatHistory {
    id: i64,
    idol_id: i32,
    msg_id: String,
    ref_msg_id: String,
    role: String,
    address: String,
    content: String,
    create_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct ChatConfig {
    chat_points: u64,
    base_fee: u64,

    #[allow(dead_code)]
    idol_pool_fee: u64,
}

// 获取地址是否是白名单
pub async fn query_chat_config(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<AIChatConfigReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_sig(headers.clone(), req.address.clone()).await?;

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&app_state.database_url)
        .await
        .map_err(new_internal_error)?;

    // TODO: 查询积分消耗配置表
    tracing::info!("=====查询积分消耗配置表");

    let query_chat_cfg_ret = sqlx::query!(
        "SELECT *
          FROM fans_config
          WHERE title = ? ",
        "PointChat"
    )
    .fetch_all(&pool)
    .await
    .map_err(new_internal_error)?;

    let chat_cfg_json = if query_chat_cfg_ret.len() > 0 {
        query_chat_cfg_ret[0]
            .content
            .clone()
            .unwrap_or(String::from(
                r#"{"chat_points":0, "base_fee":1000, "idol_pool_fee":9000}"#,
            ))
    } else {
        String::from(r#"{"chat_points":0, "base_fee":1000, "idol_pool_fee":9000}"#)
    };
    let chat_cfg: ChatConfig = serde_json::from_str(&chat_cfg_json).unwrap_or(ChatConfig {
        chat_points: 0,
        base_fee: 0,
        idol_pool_fee: 0,
    });
    tracing::info!("=====聊天消耗积分: {}", chat_cfg.chat_points);

    // 查询用户积分余额是否足够
    tracing::info!("=====查询用户积分余额");
    let query_balance_ret = sqlx::query!(
        "SELECT *
          FROM user_integral_wallet
          WHERE address = ? ",
        req.address
    )
    .fetch_all(&pool)
    .await
    .map_err(new_internal_error)?;

    let user_points_balance = if query_balance_ret.len() > 0 {
        query_balance_ret[0].balance.unwrap_or(0)
    } else {
        0
    };
    tracing::info!(
        "=====用户:{:?} , 积分余额:{:?}",
        req.address,
        user_points_balance
    );

    Ok(RespVO::from(&AIChatConfigResp {
        address: req.address,
        idol_id: req.idol_id,
        can_chat: user_points_balance as u64 > chat_cfg.chat_points,
        chat_points: chat_cfg.chat_points,
        cur_points_balance: user_points_balance as u64,
        base_fee_rate: chat_cfg.base_fee as f32 / 10000.0,
        idol_pool_rate: chat_cfg.idol_pool_fee as f32 / 10000.0,
    })
    .resp_json())
}

// ai idol聊天
pub async fn ai_chat(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<AIChatReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_sig(headers.clone(), req.address.clone()).await?;

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&app_state.database_url)
        .await
        .map_err(new_internal_error)?;
    let msg_id = Uuid::new_v4().to_string();

    // sqlx::query("SET time_zone = '+8:00'")
    //     .execute(&pool)
    //     .await
    //     .map_err(new_internal_error)?;

    // 查询积分消耗配置表
    tracing::info!("=====查询积分消耗配置表");

    let query_chat_cfg_ret = sqlx::query!(
        "SELECT *
        FROM fans_config
        WHERE title = ? ",
        "PointChat"
    )
    .fetch_all(&pool)
    .await
    .map_err(new_internal_error)?;

    let chat_cfg_json = if query_chat_cfg_ret.len() > 0 {
        query_chat_cfg_ret[0]
            .content
            .clone()
            .unwrap_or(String::from(
                r#"{"chat_points":0, "base_fee":1000, "idol_pool_fee":9000}"#,
            ))
    } else {
        String::from(r#"{"chat_points":0, "base_fee":1000, "idol_pool_fee":9000}"#)
    };
    let chat_cfg: ChatConfig = serde_json::from_str(&chat_cfg_json).unwrap_or(ChatConfig {
        chat_points: 0,
        base_fee: 0,
        idol_pool_fee: 0,
    });
    let base_fee_points = chat_cfg.base_fee * chat_cfg.chat_points / 10000; // 平台的积分抽水
    let idol_pool_points = chat_cfg.chat_points - base_fee_points;
    tracing::info!("=====聊天消耗积分: {}", chat_cfg.chat_points);

    // 查询用户积分余额是否足够
    tracing::info!("=====查询用户积分余额");
    let query_balance_ret = sqlx::query!(
        "SELECT *
        FROM user_integral_wallet
        WHERE address = ? ",
        req.address
    )
    .fetch_all(&pool)
    .await
    .map_err(new_internal_error)?;

    let user_points_balance = if query_balance_ret.len() > 0 {
        query_balance_ret[0].balance.unwrap_or(0)
    } else {
        0
    };
    tracing::info!(
        "=====用户:{:?} , 积分余额:{:?}",
        req.address,
        user_points_balance
    );

    if (user_points_balance as u64) < chat_cfg.chat_points {
        tracing::error!(
            "用户积分余额:{}, 不足聊天积分消耗:{}",
            user_points_balance,
            chat_cfg.chat_points
        );
        // return Err((
        //     StatusCode::BAD_REQUEST,
        //     Json(RespVO::<String> {
        //         code: Some(20010),
        //         msg: Some(String::from("points is not enough")),
        //         data: None,
        //     }),
        // ));

        // 这里以聊天消息的方式返回，代替报错
        return Ok(RespVO::from(&AIChatResp {
            language: req.language,
            address: req.address,
            idol_id: req.idol_id,
            msg_id: String::from(""),
            ref_msg_id: String::from(""),
            content: String::from("Sorry, your points is not enough for chatting."),
        })
        .resp_json());
    }
    // TODO: 锁定用户余额

    // 将消息插入数据库
    tracing::info!("=====将消息插入数据库");
    let _ = sqlx::query!(
        r#"
            INSERT IGNORE INTO chat_history (idol_id, msg_id, ref_msg_id, role, address, content, create_time)
            VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        req.idol_id,
        msg_id,
        "",
        "user",
        req.address,
        req.message.clone(),
        utils::datetime::get_format_datetime()
    )
    .execute(&pool)
    .await
    .map_err(new_internal_error)?
    .rows_affected();

    // 获取最近10条消息记录,作为上下文, 且保证总的长度不超过 2000个字符
    tracing::info!("=====获取最新的10条消息记录,作为聊天的上下文");
    let data = sqlx::query!(
        "SELECT *
        FROM chat_history
        WHERE idol_id = ? and address = ?
        ORDER BY create_time DESC
        LIMIT 10;",
        req.idol_id,
        req.address
    )
    .fetch_all(&pool)
    .await
    .map_err(new_internal_error)?;

    tracing::info!("data: {:?}", data);

    let mut messages: Vec<MsgBody> = Vec::new();
    for d in data {
        messages.insert(
            0,
            MsgBody {
                role: d.role,
                content: d.content.clone(),
            },
        );
    }

    let req_json_body = json!({
        "idolId": req.idol_id,
        "language": req.language,

        // 只保留最新10条消息
        "message":messages
    });

    tracing::info!("====openlove请求参数: {}", req_json_body.to_string());
    // 发起http请求
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/chat/send", app_state.openlove_url))
        .json(&req_json_body)
        .timeout(Duration::new(60, 0)) // 超时时间 60s
        .send()
        .await
        .map_err(new_internal_error)?;

    let rsp: OpenloveResp = resp.json().await.map_err(new_internal_error)?;

    let mut err_msg_map = HashMap::new();
    err_msg_map.insert(1, String::from("role is not exists"));
    err_msg_map.insert(2, String::from("upstream request failed"));
    err_msg_map.insert(3, String::from("upstream server occured error"));
    err_msg_map.insert(4, String::from("empty message is not permitted"));

    if rsp.code != 0 {
        tracing::error!("请求错误, code:{}, msg:{}", rsp.code, rsp.msg);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(20000 + rsp.code as i32),
                msg: Some(
                    err_msg_map
                        .get(&rsp.code)
                        .unwrap_or(&String::from("unknow upstream error"))
                        .clone(),
                ),
                data: None,
            }),
        ));
    }

    // 将回复插入数据库
    tracing::info!("=====将回复插入数据库=====");
    let reply_msg_id = Uuid::new_v4().to_string();
    let rsp_content = rsp.content.unwrap_or(String::from(""));
    let _ = sqlx::query!(
        r#"
            INSERT IGNORE INTO chat_history (idol_id, msg_id, ref_msg_id, role, address, content, create_time)
            VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        req.idol_id,
        reply_msg_id.clone(),
        msg_id,
        "assistant",
        req.address,
        rsp_content.clone(),
        utils::datetime::get_format_datetime()
    )
    .execute(&pool)
    .await
    .map_err(new_internal_error)?
    .rows_affected();

    // 扣减积分
    tracing::info!("=====扣减积分=====");
    let _ = sqlx::query!(
        r#"
            UPDATE user_integral_wallet
            SET
            balance = balance - ?
            WHERE
            address = ?
            AND
            balance >= ?
            "#,
        chat_cfg.chat_points,
        req.address,
        chat_cfg.chat_points,
    )
    .execute(&pool)
    .await
    .map_err(new_internal_error)?
    .rows_affected();

    // 插入积分消耗记录
    tracing::info!("=====插入积分消耗记录======");
    let _ = sqlx::query!(
        r#"
            INSERT IGNORE INTO ai_idol_point_record (id, idol_id, amount, address, base_fee, idol_pool_fee, trans_type, create_time)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        msg_id.clone(),
        req.idol_id,
        chat_cfg.chat_points, // 消耗的积分
        req.address,
        base_fee_points, // 平台手续费获得的积分
        idol_pool_points, // 偶像积分池的积分
        11, // 聊天消耗
        utils::datetime::get_format_datetime()
    )
    .execute(&pool)
    .await
    .map_err(new_internal_error)?
    .rows_affected();

    // 插入流水记录
    tracing::info!("=====插入聊天积分消耗流水记录======");
    let cur_timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("获取时间戳失败")
    .as_micros().to_string() // 16位
    + &rand::thread_rng().gen_range(1111111..=9999999).to_string();

    // 获取当前时间戳
    let current_time = Local::now();
    // current_time.with_timezone(TimeZone::);
    // 将时间戳转换为 UTC+8 时区
    // let fixed_offset = FixedOffset::(8 * 3600); // UTC+8 时区偏移量，单位为秒
    // let local_time:  = current_time.with_timezone(&fixed_offset).to;

    tracing::info!("cur time: {}", current_time.to_string());

    let _ = sqlx::query!(
        r#"
            INSERT IGNORE INTO events_integral_record (id, events_id,  address, amount, `decimal`, type, create_time, hash)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        &cur_timestamp[0..18],
        req.idol_id,
        req.address,
        chat_cfg.chat_points, // 消耗的积分
        0,
        11, // 聊天消耗
        // "2024-04-29 11:55:00",
        utils::datetime::get_format_datetime(),
        msg_id.clone(),
    )
    .execute(&pool)
    .await
    .map_err(new_internal_error)?
    .rows_affected();

    Ok(RespVO::from(&AIChatResp {
        language: req.language,
        address: req.address,
        idol_id: req.idol_id,
        msg_id: reply_msg_id,
        ref_msg_id: msg_id.clone(),
        content: rsp_content.clone(),
    })
    .resp_json())
}

// 查询聊天记录
pub async fn query_chat_history(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<AIChatHistoryReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_sig(headers.clone(), req.address.clone()).await?;

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&app_state.database_url)
        .await
        .map_err(new_internal_error)?;

    let limit = if req.page_size > 0 { req.page_size } else { 10 };
    let offset = req.page * req.page_size;

    // 分页获取历史消息
    let query = sqlx::query_as::<_, ChatHistory>(
        "SELECT *
        FROM chat_history
        WHERE idol_id = ? and address = ?
        ORDER BY create_time DESC
        LIMIT ?
        OFFSET ?
        ;",
    )
    .bind(req.idol_id)
    .bind(req.address.clone())
    .bind(limit)
    .bind(offset);

    let data = query.fetch_all(&pool).await.map_err(new_internal_error)?;

    tracing::info!("data: {:?}", data);

    let mut messages: Vec<HistoryChatMsgBody> = Vec::new();
    for d in data {
        messages.push(HistoryChatMsgBody {
            no: d.id,
            msg_id: d.msg_id,
            idol_id: d.idol_id as u32,
            role: d.role,
            ref_msg_id: d.ref_msg_id,
            content: d.content.clone(),
            timestamp: d.create_time.to_rfc3339(),
        });
    }

    Ok(RespVO::from(&AIChatHistoryResp {
        address: req.address.clone(),
        idol_id: req.idol_id,
        language: req.language,
        page: req.page,
        page_size: req.page_size,
        // total_count: 0,
        history_messages: messages,
    })
    .resp_json())
}

// 获取地址积分记录
#[allow(dead_code)]
pub async fn get_address_points_history(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryAddressPointsHistoryReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    // let _ = verify_sig(headers.clone(), req.address.clone()).await?;
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

    // 使用redis
    let mut rds_conn = app_state
        .rds_pool
        .aquire()
        .await
        .map_err(new_internal_error)?;

    // 查询地址积分
    // 通过命令: ZREVRANGEBYSCORE points:0x51bdbad59a24207b32237e5c47e866a32a8d5ed8 9999999999 0 WITHSCORES
    let point_prefix_key = String::from("points:") + &req.address.to_lowercase();

    // 分页
    let offset = (req.page * req.page_size) as isize;
    let count = req.page_size as isize;

    tracing::info!("{point_prefix_key}");

    // 获取总条数
    let total_counts: Vec<u32> = redis::pipe()
        .zcard(&point_prefix_key)
        .query_async(&mut rds_conn)
        .await
        .map_err(new_internal_error)?;

    let mut historys: Vec<Point> = Vec::new();
    if req.page_size > 0 {
        let points_ret: Vec<Vec<String>> = redis::pipe()
            .zrevrangebyscore_limit(&point_prefix_key, 9999999999_u64, 0, offset, count)
            .query_async(&mut rds_conn)
            .await
            .map_err(new_internal_error)?;

        tracing::info!("points history: {:?}", points_ret);

        if points_ret.len() > 0 && points_ret[0].len() > 0 {
            tracing::info!("points history length: {:?}", points_ret[0].len());
            for item in &points_ret[0] {
                let parts: Vec<&str> = item.split('_').collect();
                historys.push(Point {
                    chain_id: parts[0].parse().unwrap(),
                    value: parts[1].parse().unwrap(),
                    method: parts[2].to_owned(),
                    timestamp: parts[3].parse().unwrap(),
                    txhash: parts[4].to_owned(),
                });
            }
        }
    }

    Ok(RespVO::from(&QueryAddressPointsHistoryResp {
        address: req.address,
        total: total_counts[0],
        page: req.page,
        page_size: req.page_size,
        history: historys,
    })
    .resp_json())
}

pub async fn bind_email(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(req): JsonReq<BindEmailReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let _ = verify_token(headers, &app_state, req.address.clone()).await?;

    // 验证邮箱地址是否合法
    if !EmailAddress::is_valid(&req.email) {
        tracing::warn!("邮箱地址非法:{}", req.email);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10013),
                msg: Some("invalid email".to_owned()),
                data: None,
            }),
        ));
    }

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

    let mut msg_domain = app_state.web_domain;
    if req.is_ai.unwrap_or(false) {
        msg_domain = match app_state.env.to_uppercase().as_str() {
            "TEST" => "test-ai.fansland.xyz".to_owned(),
            "UAT" => "uat-ai.fansland.io".to_owned(),
            "PRO" => "ai.fansland.io".to_owned(),
            _ => "ai.fansland.io".to_owned(),
        }
    }
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
            + 22 * 3600,
    );
    // 1: web2 , 2: web3 ， 固定是2即可
    jwt_token.set_user_type(2);

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

    let k = format!("FANSLAND_NFT_{}_{}", req.chainid, app_state.env);
    tracing::debug!("k = {}", k);
    let fansland_nft_contract_address = std::env::var(k).map_err(new_internal_error)?;

    let acc_type = match req.access_type {
        Some(x) => match x {
            2 => 2,
            3 => 3,
            _ => 3,
        },
        None => 3,
    };

    query_ticket_qrcode_by_token_id(
        rds_conn,
        fansland_nft_contract_address,
        req.address.clone(),
        req.token_id,
        req.chainid,
        acc_type,
    )
    .await
}

pub async fn query_ticket_qrcode_by_token_id(
    mut rds_conn: RedisPoolConnection<Connection>,
    contract_address: String,
    address: String,
    token_id: u32,
    chainid: u64,
    access_type: u8,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    tracing::debug!("=从redis获取绑定tokenid对应的二维码, chainid:{chainid}, adddress:{address}, token_id:{token_id} ",);
    // 2024-03-28
    // 如果是web2用户，用兑换码兑换，立即来查看, 直接可以查看二维码
    // 设置： SADD redeemaddress {chainId}_{tokenId}_{userAddress}
    // 判断是否在集合:  SISMEMBER redeemaddress {chainId}_{tokenId}_{userAddress}
    // let mut token_id_owner = address;

    // access_type 访问类型:  3: web3 ,  2: web2
    let is_web2_redeem = false;

    // ==== 关闭 web2查二维码， 所有web2二维码提前生成好，导入李咏那边，都走李咏那边
    // if access_type == 2 {
    //     let redeemaddress_key = "redeemaddress";
    //     let member = format!("{}_{}_{}", chainid, token_id, address.to_lowercase());
    //     let is_redeem_ret: Vec<u64> = redis::pipe()
    //         .sismember(redeemaddress_key, member)
    //         .query_async(&mut rds_conn)
    //         .await
    //         .map_err(new_internal_error)?;

    //     is_web2_redeem = is_redeem_ret[0] == 1;

    //     if is_web2_redeem {
    //         tracing::info!("Good, 用户存在兑换码兑换记录, 走web2的业务逻辑,直接返回二维码");
    //     } else {
    //         tracing::info!("Sorry, 不存在兑换记录, 要走web3的业务逻辑, 校验链上数据情况");
    //     }
    // }

    if !is_web2_redeem {
        // 从redis中获取该token_id的owner
        let key_prefix = "nft:".to_owned()
            + &chainid.to_string()
            + ":nft:tokenid:owner:"
            + &token_id.to_string();
        let token_id_owner = match redis::cmd("GET")
            .arg(&key_prefix)
            .query_async::<_, Option<String>>(&mut rds_conn)
            .await
            .map_err(new_internal_error)?
        {
            Some(m) => m,
            None => "".to_owned(),
        };

        // 比对redis中的owner与参数中的owner是否相同
        if !token_id_owner.to_lowercase().eq(&address.to_lowercase()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(RespVO::<String> {
                    code: Some(11001),
                    msg: Some("The NFT ticket is pending. Please wait a few minutes.".to_owned()),
                    data: None,
                }),
            ));
        }
    }

    // 根据算法生成二维码
    let mut fansland_nft_contract_address = contract_address;
    fansland_nft_contract_address = fansland_nft_contract_address.to_lowercase();
    // FIX: for fix , 之前BSC早鸟票使用了大写, 使用这种方式修补, 2024-3-25 by yqq
    if fansland_nft_contract_address.to_lowercase() == "0xbf36ab3aed81bf8553b52c61041904d98ee882c2"
    {
        fansland_nft_contract_address = String::from("0xBf36aB3AeD81Bf8553B52c61041904d98Ee882C2");
    }

    let salt = "QrCode@fansland.io2024-888"; // TODO:
    let hash_msg = String::new()
        + &fansland_nft_contract_address
        + &token_id.to_string()
        + &address.to_lowercase()
        + &chainid.to_string()
        + salt;
    let keccak_hash = ethers::utils::keccak256(hash_msg.as_bytes());
    let bz_qrcode = &keccak_hash[keccak_hash.len() - 15..];
    let qrcode = String::from("1:") + &hex::encode(bz_qrcode);

    tracing::info!("qrcode: {}", qrcode);

    let r = QueryTicketQrCodeResp {
        user_address: address,
        nft_token_id: token_id,
        qrcode: qrcode,
        contract_address: fansland_nft_contract_address,
        chain_id: chainid,
    };

    Ok(RespVO::from(&r).resp_json())
}

// 私密链接查询门票二维码
pub async fn get_ticket_qrcode_by_secret_link(
    headers: HeaderMap,
    State(app_state): State<AppState>,
    JsonReq(secret_token_req): JsonReq<GetTicketQrCodeBySecretTokenReq>,
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

    let fansland_nft_contract_address =
        std::env::var(format!("FANSLAND_NFT_{}_{}", req.chainid, app_state.env)).unwrap();

    // 查询门票二维码
    // 私密链接访问， access_type 默认为 3， 即 web3用户
    query_ticket_qrcode_by_token_id(
        rds_conn,
        fansland_nft_contract_address,
        req.address.clone(),
        req.token_id,
        req.chainid,
        3,
    )
    .await
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
    // check address
    if !(address.len() == 42 && address.starts_with("0x")) {
        tracing::error!("====地址格式错误:{}==========", address);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(-1),
                msg: Some("bad request".to_string()),
                data: None,
            }),
        ));
    }

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
    if now_ts.abs_diff(api_timestamp) > 10_000 {
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
// 10013: invalid email 邮箱地址非法
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
