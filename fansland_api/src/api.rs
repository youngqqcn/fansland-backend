// use super::schema::*;
// use chrono::NaiveDateTime;
// use diesel::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryAddressReq {
    pub address: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct BindEmailReq {
    pub address: String,
    pub email: String,
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct BindEmailResp {
    pub success: bool,
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
    pub token: String, // token
}

// get wallet login nonce
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct GetLoginNonceResp {
    pub address: String,
    pub signmsg: String,
}

// get tickets by secret token
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct GetTicketsBySecretToken {
    pub address: String, // 地址
    pub token: String,   // token
    pub passwd: String,  // 密码hash
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
