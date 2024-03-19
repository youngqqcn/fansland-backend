use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct QueryTicketQrCodeReq {
    pub chain_id: u64, // chainid
    pub nft_contract: String, // 合约地址
    pub nft_owner: String, // NFT持有者地址
    pub nft_token_id: u64, // token_id

    pub timestamp: u64, // 当前时间戳
    pub signature: String, // 签名
}

#[derive(Default, Deserialize, Clone, Debug, Serialize)]
pub struct QueryTicketQrCodeResp {
    pub qrcode: String,
    pub chain_id: u64,
    pub nft_contract: String,
    pub nft_token_id: u64,
    pub nft_owner: String,
    // pub redeem_status: u32,
    // pub type_id: u32,
    // pub ticket_type_name: String,
    // pub ticket_price: String,
    // pub event_name: String,
    // pub event_time: String,
}
