// @generated automatically by Diesel CLI.

diesel::table! {
    tickets (id) {
        id -> Int4,
        user_id -> Int8,
        user_address -> Varchar,
        chain_name -> Varchar,
        contract_address -> Varchar,
        nft_token_id -> Int8,
        txhash -> Varchar,
        qrcode -> Nullable<Varchar>,
        redeem_status -> Int4,
        transfer_status -> Int4,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        user_address -> Varchar,
        email -> Varchar,
        nonce -> Varchar,
        token -> Varchar,
        passwd -> Nullable<Varchar>,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    tickets,
    users,
);
