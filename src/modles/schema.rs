// @generated automatically by Diesel CLI.

diesel::table! {
    tb_events (id) {
        id -> Int8,
        ev_name -> Nullable<Varchar>,
        ev_desc -> Nullable<Varchar>,
        ev_banner_img_url -> Nullable<Varchar>,
        contract_address -> Nullable<Varchar>,
    }
}

diesel::table! {
    tb_ticket_types (id) {
        id -> Int8,
        name -> Varchar,
        price -> Int4,
        contract_type_id -> Int4,
    }
}

diesel::table! {
    tb_user_tickets (id) {
        id -> Int8,
        user_id -> Int8,
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
    tb_users (id) {
        id -> Int8,
        address -> Varchar,
        email -> Varchar,
        nonce -> Varchar,
        token -> Varchar,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    tb_events,
    tb_ticket_types,
    tb_user_tickets,
    tb_users,
);
