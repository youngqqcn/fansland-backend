-- Your SQL goes here
-- -- up.sql

CREATE TABLE users (
    id SERIAL NOT NULL,
    user_address varchar NOT NULL,
    email varchar NOT NULL,
    nonce varchar not null,
    token varchar not null,
    passwd varchar null,
    update_at timestamp(6) default current_timestamp,
    CONSTRAINT user_pk PRIMARY KEY (id),
    CONSTRAINT uidx_address UNIQUE (user_address),
    CONSTRAINT uidx_email UNIQUE (email)
);
INSERT INTO public.users
(id, user_address, email, nonce, "token", passwd, update_at)
VALUES(1, '0xbfe5f435389ca190c3d3dec351db0ee9a8657a53', 'youngqqcn@gmail.com', '1234', 'happy', '6666', '2024-01-17 10:49:12.776');


CREATE TABLE tickets (
    id SERIAL NOT NULL,
    user_id int8 not null,
    user_address varchar not null,
    chain_name varchar not null,
    contract_address varchar not null,
    nft_token_id int8 not null,
    txhash varchar not null,
    qrcode varchar NULL,
    redeem_status int not null,
    transfer_status int not null,
    ticket_type_id int4 not null,
    ticket_type_name varchar not null,
    ticket_price int4 not null,
    event_name varchar not null,
    event_time varchar not null,
    update_at timestamp(6) default current_timestamp,
    CONSTRAINT tb_user_tickets_pk PRIMARY KEY (id),
    CONSTRAINT uidx_contract_tokenid UNIQUE (contract_address, nft_token_id)
);
CREATE INDEX idx_user_address ON tickets USING btree (user_address);



INSERT INTO public.tickets (
        id,
        user_id,
        user_address,
        chain_name,
        contract_address,
        nft_token_id,
        txhash,
        qrcode,
        redeem_status,
        transfer_status,
        ticket_type_id,
        ticket_type_name,
        ticket_price,
        event_name,
        event_time,
        update_at
    )
VALUES(
        1,
        1,
        '0xaaaa',
        'polygon',
        '0xaaaaaa',
        1,
        '0xaaaaaaaaa',
        'skflsfdjlsf',
        0,
        0,
        0,
        '3 Days Ticket',
        9900,
        'Fansland Web3 Music Festival 2024 Thailand Bangkok',
        '4-6 May 2024',
        '2024-01-16 11:11:29.436'
    );
INSERT INTO public.tickets (
        id,
        user_id,
        user_address,
        chain_name,
        contract_address,
        nft_token_id,
        txhash,
        qrcode,
        redeem_status,
        transfer_status,
        ticket_type_id,
        ticket_type_name,
        ticket_price,
        event_name,
        event_time,
        update_at
    )
VALUES(
        2,
        2,
        '0xcccc',
        'eth',
        '0xbbbb',
        2,
        '0xbbbb',
        'sflsdfksdlfjl',
        0,
        0,
        0,
        '2 Days Ticket',
        9900,
        'Fansland Web3 Music Festival 2024 Thailand Bangkok',
        '4-6 May 2024',
        '2024-01-16 11:38:24.163'
    );