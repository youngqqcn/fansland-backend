// use super::schema::*;
// use chrono::NaiveDateTime;
// use diesel::prelude::*;

#[derive(serde::Deserialize, Clone)]
pub struct BindEmailReq {
    pub address: String,
    pub email: String,
}

#[derive(serde::Serialize, Clone)]
pub struct BindEmailResp {
    pub success: bool,
}
