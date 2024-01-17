use super::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Deserialize;

#[derive(serde::Serialize, Selectable, Queryable, Clone, Deserialize, Debug)]
pub struct User {
    pub id: i32,
    pub user_address: String,
    pub email: String,
    pub nonce: String,
    pub token: String,
    pub passwd: Option<String>,
    pub update_at: Option<NaiveDateTime>,
}

#[derive(serde::Deserialize, Insertable, Clone)]
#[diesel(table_name = users)]
pub struct CreateUser {
    // pub id: i32,
    pub user_address: String,
    pub email: String,
    pub nonce: String,
    pub token: String,
    // pub update_at: Option<NaiveDateTime>,
}

// ticket
// #[derive(serde::Serialize, Selectable, Queryable, Insertable, Debug, Clone)]
#[derive(serde::Serialize, Selectable, Queryable, Insertable, Clone, Deserialize, Debug)]
pub struct Ticket {
    pub id: i32,
    pub user_id: i64,
    pub user_address: String,
    pub chain_name: String,
    pub contract_address: String,
    pub nft_token_id: i64,
    pub txhash: String,
    pub qrcode: Option<String>,
    pub redeem_status: i32,
    pub transfer_status: i32,
    pub ticket_type_id: i32,
    pub ticket_type_name: String,
    pub ticket_price: i32,
    pub event_name: String,
    pub event_time: String,
    pub update_at: Option<NaiveDateTime>,
}
