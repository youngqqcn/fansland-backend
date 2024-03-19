use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    http::StatusCode,
    response::{Json, Response},
};
use fansland_common::RespVO;
use sha256::digest;

use crate::{
    api::{QueryTicketQrCodeReq, QueryTicketQrCodeResp},
    extract::JsonReq,
};

pub async fn get_nft_ticket_qrcode(
    // headers: HeaderMap,
    // State(app_state): State<AppState>,
    JsonReq(req): JsonReq<QueryTicketQrCodeReq>,
) -> Result<Response<Body>, (StatusCode, Json<RespVO<String>>)> {
    let okx_salt = "ca17a3e225a85a74290831f504aceec5";
    let sig_msg = req.chain_id.to_string()
        + "&"
        + &req.nft_contract
        + "&"
        + &req.nft_owner
        + "&"
        + &req.nft_token_id.to_string()
        + "&"
        + &req.timestamp.to_string()
        + "&"
        + okx_salt
        ;

    // 接口签名验签
    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // 正负60秒， 即时间窗口有120s
    if now_ts.abs_diff(req.timestamp) > 60 {
        tracing::error!(
            "====时间戳无效=====系统当前时间戳: {}, 参数时间戳: {}",
            now_ts,
            req.timestamp
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RespVO::<String> {
                code: Some(10002),
                msg: Some("invalid timestamp".to_owned()),
                data: None,
            }),
        ));
    }

    tracing::debug!("sig_msg is {}", sig_msg);
    let h = digest(sig_msg.to_lowercase());
    tracing::debug!("signatrue is {}", h);

    if req.signature != h {
        tracing::error!("====签名不匹配: 期望: {} , 实际: {}", h, req.signature);

        return Err((
            StatusCode::BAD_REQUEST,
            axum::Json(RespVO::<String> {
                code: Some(10000),
                msg: Some(String::from("invalid signature")),
                data: None,
            }),
        ));
    }

    let salt = "QrCode@fansland.io2024-888";
    let hash_msg = String::new()
        + &req.chain_id.to_string()
        + &req.nft_contract
        + &req.nft_token_id.to_string()
        + &req.nft_owner
        + salt;
    let keccak_hash = ethers::utils::keccak256(hash_msg.as_bytes());
    let bz_qrcode = &keccak_hash[keccak_hash.len() - 15..];
    let qrcode = String::from("1:") + &hex::encode(bz_qrcode);

    let r = QueryTicketQrCodeResp {
        chain_id: req.chain_id,
        qrcode: qrcode,
        nft_owner: req.nft_owner,
        nft_contract: req.nft_contract,
        nft_token_id: req.nft_token_id,
    };

    Ok(RespVO::from(&r).resp_json())
}
