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
pub struct GetTicketQrCodeBySecretToken {
    pub address: String, // 地址
    pub token: String,  // token
    pub passwd: String, // 密码hash
    pub token_id: u32, // token
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryTicketQrCodeReq {
    pub address: String,
    pub token_id: u32, // token_id
}

#[derive(Default, Deserialize, Clone, Debug, Serialize)]
pub struct QueryTicketQrCodeResp {
    pub user_address: String,
    // pub chain_name: String,
    // pub contract_address: String,
    pub nft_token_id: u32,
    pub qrcode: String,
    // pub redeem_status: u32,
    // pub type_id: u32,
    // pub ticket_type_name: String,
    // pub ticket_price: String,
    // pub event_name: String,
    // pub event_time: String,
}
