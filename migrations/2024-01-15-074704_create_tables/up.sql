-- Your SQL goes here
-- -- up.sql
-- CREATE TABLE "users"(
--     "id" SERIAL PRIMARY KEY,
--     "name" TEXT NOT NULL,
--     "hair_color" TEXT
-- );
CREATE TABLE tb_users (
    id int8 NOT NULL,
    ---comment "id",
    "address" varchar NOT NULL,
    ---comment "用户钱包",
    email varchar NOT NULL,
    ---comment "用户邮箱",
    nonce varchar not null,
    ---comment "登录nonce"
    token varchar not null,
    ---comment "登录token",
    update_at timestamp(6) default current_timestamp,
    ---comment "更新时间",
    CONSTRAINT user_pk PRIMARY KEY (id),
    CONSTRAINT uidx_address UNIQUE (address),
    CONSTRAINT uidx_email UNIQUE (email)
);
CREATE TABLE tb_events (
    id int8 NOT NULL,
    ---comment "id",
    ev_name varchar NULL,
    ---comment "活动名称",
    ev_desc varchar NULL,
    ---comment "活动描述",
    ev_banner_img_url varchar NULL,
    ---comment "图片链接",
    contract_address varchar NULL,
    ---comment "合约地址",
    CONSTRAINT tb_event_pk PRIMARY KEY (id)
);
CREATE TABLE tb_ticket_types (
    id int8 NOT NULL,
    ---comment "id",
    "name" varchar NOT NULL,
    ---comment "类型名",
    price int4 NOT NULL,
    ---comment "价格",
    contract_type_id int4 NOT NULL,
    ---comment "合约中对应的typeid",
    CONSTRAINT tb_ticket_type_pk PRIMARY KEY (id)
);

CREATE TABLE tb_user_tickets (
    id int8 NOT NULL,
    ---comment "id",
    user_id int8 not null,
    ---comment "用户id",
    chain_name varchar not null,
    ---comment "链名称",
    contract_address varchar not null,
    ---comment "NFT合约地址",
    nft_token_id int8 not null,
    ---comment "NFT tokenid",
    txhash varchar not null,
    ---comment "transfer交易hash",
    qrcode varchar NULL,
    ---comment "二维码",
    redeem_status int not null,
    ---comment "核销状态: 0:未消耗, 1:已核销, 2:已过期",
    transfer_status int not null,
    ---comment "转移状态: 0:未转移， 1:已转移",
    update_at timestamp(6) default current_timestamp,
    ---comment "更新时间",
    CONSTRAINT tb_user_tickets_pk PRIMARY KEY (id)
);