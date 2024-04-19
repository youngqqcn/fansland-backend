use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryAddressReq {
    pub address: String,
}

#[derive(Default, Deserialize, Clone, Debug, Serialize)]
pub struct QueryAddressResp {
    pub address: String,
    pub email: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct BindEmailReq {
    pub address: String,
    pub email: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct BindEmailResp {
    pub success: bool,
    pub address: String,
    pub email: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct LoginByAddressReq {
    pub address: String,
    pub msg: String,
    pub sig: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct LoginByAddressResp {
    pub success: bool,
    pub address: String,
    pub token: String, // token
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct GetLoginNonceReq {
    pub address: String,
    pub chainid: u64,
}

// get wallet login nonce
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct GetLoginNonceResp {
    pub address: String,
    pub signmsg: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct UpdateSecretLinkPasswdReq {
    pub address: String, // 地址
    pub passwd: String,  // 密码hash
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct UpdateSecretLinkPasswdResp {
    pub success: bool,
    pub secret_token: String,
}

// get tickets by secret token
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct GetTicketQrCodeBySecretTokenReq {
    pub address: String, // 地址
    pub token: String,   // token
    pub passwd: String,  // 密码hash
    pub token_id: u32,   // token
    pub chainid: u64,    // chainid
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryTicketQrCodeReq {
    pub address: String,
    pub token_id: u32, // token_id
    pub chainid: u64,  // chainid

    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_type: Option<u8>, // 访问类型:  2: web2  3: web3
}

#[derive(Default, Deserialize, Clone, Debug, Serialize)]
pub struct QueryTicketQrCodeResp {
    pub user_address: String,
    pub chain_id: u64,
    pub contract_address: String,
    pub nft_token_id: u32,
    pub qrcode: String,
    // pub redeem_status: u32,
    // pub type_id: u32,
    // pub ticket_type_name: String,
    // pub ticket_price: String,
    // pub event_name: String,
    // pub event_time: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryAddressPointsReq {
    pub address: String,
    // pub chain_id: u64, // 0: 查询所有 , 非0：查询指定链的积分
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct Point {
    pub chain_id: u64,
    pub value: u64,
    pub method: String,
    pub timestamp: u64,
    pub txhash: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryAddressPointsResp {
    pub address: String,
    pub points: u32,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryAddressPointsHistoryReq {
    pub address: String,
    // pub chain_id: u64, // 0: 查询所有 , 非0：查询指定链的积分
    pub page: u32,
    pub page_size: u32,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryAddressPointsHistoryResp {
    pub address: String,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub history: Vec<Point>,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryPointsRankReq {
    pub page: u32,
    pub page_size: u32,
    pub address: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct Rank {
    pub rank_no: u32,
    pub address: String,
    pub points: u32,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryPointsRankResp {
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub rank: Vec<Rank>,
}

// 检查白名单请求
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryWhitelistReq {
    pub address: String,
}

// 检查白名单响应
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryWhitelistResp {
    pub address: String,
    pub is_whitelist: bool,
}

// AI聊天请求
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct AIChatReq {
    pub address: String,
    pub idol_id: u32,
    pub language: String,
    pub message: String,
}

// AI聊天响应
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct AIChatResp {
    pub address: String,
    pub idol_id: u32,
    pub language: String,
    pub msg_id: String,
    pub ref_msg_id: String,
    pub content: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct MsgBody {
    pub role: String,
    pub content: String,
}

// 请求 openlove
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct OpenloveReq {
    pub idol_id: u32,
    pub language: String,
    pub messages: Vec<MsgBody>,
}

// openlove的响应
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct OpenloveResp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    pub code: u32,
    pub success: bool,
    pub msg: String,
}

// 查询历史聊天记录
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct AIChatHistoryReq {
    pub address: String,
    pub idol_id: u32,
    pub language: String,
    pub page: u32,
    pub page_size: u32,
}


#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct HistoryChatMsgBody {
    pub no: i64,
    pub msg_id: String,
    pub idol_id: u32,
    pub role: String,
    pub ref_msg_id: String, // 如果是AI的回复,此字段就是指向用户的发起的msg_id
    pub content: String,
    pub timestamp: String,
}

// 查询历史聊天记录响应
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct AIChatHistoryResp {
    pub address: String,
    pub idol_id: u32,
    pub language: String,
    pub page: u32,
    pub page_size: u32,
    // pub total_count: u32,
    pub history_messages: Vec<HistoryChatMsgBody>,
}
