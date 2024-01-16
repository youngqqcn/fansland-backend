use super::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(serde::Serialize, Selectable, Queryable)]
pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
}

#[derive(serde::Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    name: String,
    hair_color: Option<String>,
}

// ticket
#[derive(serde::Serialize, Selectable, Queryable)]
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


