use super::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(serde::Serialize, Selectable, Queryable)]
pub struct User {
    pub id: i32,
    pub address: String,
    pub email: String,
    pub nonce: String,
    pub token: String,
    pub update_at: Option<NaiveDateTime>,
}

#[derive(serde::Deserialize, Clone)]
pub struct BindEmail {
    // pub id: i32,
    pub address: String,
    pub email: String,
}

#[derive(serde::Deserialize, Insertable, Clone)]
#[diesel(table_name = users)]
pub struct CreateUser {
    // pub id: i32,
    pub address: String,
    pub email: String,
    pub nonce: String,
    pub token: String,
    // pub update_at: Option<NaiveDateTime>,
}

// ticket
#[derive(serde::Serialize, Selectable, Queryable, Insertable, Debug)]
pub struct Ticket {
    id: i32,
    user_id: i64,
    chain_name: String,
    contract_address: String,
    nft_token_id: i64,
    txhash: String,
    qrcode: Option<String>,
    redeem_status: i32,
    transfer_status: i32,
    update_at: Option<NaiveDateTime>,
}
