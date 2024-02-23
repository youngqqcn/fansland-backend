use clap::Parser;
use ctrlc;
use dotenv::dotenv;
use ethers::{
    abi::AbiEncode,
    contract::abigen,
    core::types::{Address, Filter, U256},
    providers::{Http, Middleware, Provider},
    types::H256,
};
use redis::Client;
use redis_pool::RedisPool;
use std::time::Duration;
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::time::sleep;
use tracing::Level;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// chainid , [80001: polygon-mumbai testnet] , [97: polygon-pos mainnet], [137: bsc-testnet], [56: bsc-mainnet]
    #[arg(short, long)]
    chainid: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let args = Args::parse();

    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
    //     )
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();
    // test().await?;

    let chainid: u64 = args.chainid;
    let rpc_url = std::env::var(format!("RPC_URL_{chainid}")).unwrap();
    let contract_address = std::env::var(format!("FANSLAND_NFT_{chainid}")).unwrap();
    let contract_create_block: u64 = std::env::var(format!("CONTRACT_CREATE_BLOCK_{chainid}"))
        .unwrap()
        .parse()?;

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // 创建一个原子布尔变量来表示是否收到了 SIGKILL 信号
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // 设置信号处理程序
    ctrlc::set_handler(move || {
        running.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // 在主循环中执行操作，直到收到 SIGKILL 信号
    tracing::info!("Running...");
    while r.load(Ordering::SeqCst) {
        // 执行你的操作
        let _ = update_token_id_owner(&rpc_url, &contract_address, chainid, contract_create_block)
            .await;
        // 睡眠一段时间，然后继续下一次循环
        for _ in 0..10 {
            sleep(Duration::from_secs(1)).await;
            if !r.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    println!("Received SIGKILL. Exiting...");

    Ok(())
}

// 更新token的owner
async fn update_token_id_owner(
    rpc_url: &String,
    contract_address: &String,
    chainid: u64,
    contract_create_block: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(rpc_url)?;
    abigen!(FanslandNFTContract, "FanslandNFT.abi",);
    tracing::info!("nft_contract address: {}", contract_address);
    tracing::info!("rpc_url : {}", rpc_url);

    let client = Arc::new(provider);

    let fansland_nft_contract_address = contract_address;
    let contract_address_h160: Address = contract_address.parse()?;
    let contract = FanslandNFTContract::new(contract_address_h160, client.clone());

    let rds_url = std::env::var("REDIS_URL").unwrap();
    tracing::debug!("rds_url: {}", rds_url);
    let rds_client = Client::open(rds_url).unwrap();
    let redis_pool = RedisPool::from(rds_client);
    let mut rds_conn = redis_pool.aquire().await?;

    // nft相关的前缀
    let key_prefix = "nft:".to_owned() + &chainid.to_string() + ":";

    // 积分相关前缀, 统一放在一起，不根据链类型分, 方便统计
    let points_prefix = "points:".to_owned();

    // 获取当前数据库中的扫描起始高度
    let scan_from_block: u64 = match redis::cmd("GET")
        .arg(key_prefix.clone() + "cur_scan_block")
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await?
    {
        Some(m) => m.parse()?,
        None => contract_create_block, // 如果获取不到，则从合约创建的区块高度开始
    };

    // 获取当前最新高度
    let latest_block = client.get_block_number().await?;
    let scan_to_block = latest_block.as_u64(); // 不等区块确认
    if scan_to_block <= scan_from_block {
        // 不用扫
        tracing::info!("已扫到最新区块:{scan_to_block}");
        return Ok(());
    }

    tracing::info!("ChainID-{chainid}扫描区块范围:{scan_from_block} - {scan_to_block}");

    let mut tmp_from_block = scan_from_block;
    loop {
        // 控制步长，步子不能太大，有些RPC不支持超过1000
        let tmp_to_block = std::cmp::min(tmp_from_block + 866, scan_to_block);
        if tmp_from_block >= tmp_to_block {
            break;
        }

        // NFT合约签名 Topic0
        // Transfer(address,address,uint256): 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
        // MintNft(address,address,uint256,uint256): 0x2272eb210d3656f000e10b00f8a373a14c45c835a3312455e0f6127d63011563
        let transfer_topic0 =
            H256::from_str("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")
                .unwrap();
        // let mint_nft_topic0 =
        //     H256::from_str("0x2272eb210d3656f000e10b00f8a373a14c45c835a3312455e0f6127d63011563").unwrap();

        let filter = Filter::new()
            .address(fansland_nft_contract_address.parse::<Address>()?)
            // .event("Transfer(address,address,uint256)")
            .events(vec![
                "Transfer(address,address,uint256)",
                "MintNft(address,address,uint256,uint256)",
            ])
            .from_block(tmp_from_block)
            .to_block(tmp_to_block);

        let logs = client.get_logs(&filter).await?;
        tracing::info!("{} Transfer events found!", logs.iter().len());
        for log in logs.iter() {
            tracing::info!("txhash: {}", &log.transaction_hash.unwrap().encode_hex());
            // Transfer evnet
            tracing::info!("topic 0 : {}", (log.topics[0]).encode_hex());
            if log.topics[0].eq(&transfer_topic0) {
                let from_addr_h160 = Address::from(log.topics[1]);
                let to_addr_h160 = Address::from(log.topics[2]);

                let from_addr_hex =
                    "0x".to_string() + &hex::encode(from_addr_h160.to_fixed_bytes());
                let to_addr_hex = "0x".to_string() + &hex::encode(to_addr_h160.to_fixed_bytes());

                let token_id = U256::from_big_endian(&log.topics[3].as_bytes()[0..32]);
                tracing::info!(
                    "Transfer event: from_addr = {from_addr_hex}, to_addr = {to_addr_hex}, token_id= {token_id}"
                );

                // 获取nft票的类型
                let type_id: u64 = contract.token_id_type_map(token_id).call().await?.as_u64();

                let qrcode_txt =
                    gen_qrcode_text(chainid, token_id.as_u64(), to_addr_hex.to_lowercase());
                tracing::info!("token_id qrcode: {}", qrcode_txt);

                // 插入数据库
                let _ = redis::pipe()
                    .set(
                        key_prefix.clone() + &format!("nft:tokenid:owner:{token_id}"),
                        &to_addr_hex,
                    )
                    .set(
                        key_prefix.clone() + &format!("nft:tokenid:type:{token_id}"),
                        type_id,
                    )
                    .sadd(key_prefix.clone() + "tokenids", token_id.as_u64())
                    .sadd(key_prefix.clone() + "holders", &to_addr_hex)
                    .rpush(
                        "sendemail",
                        &format!(
                            "{};{};{};{};{}",
                            chainid,
                            to_addr_hex.clone().to_owned(),
                            token_id,
                            type_id,
                            qrcode_txt
                        ),
                    )
                    // .ignore()
                    .query_async(&mut rds_conn)
                    .await?;
            } else {
                // MintNft event
                let user_addr_h160 = Address::from(log.topics[1]);
                let kol_addr_h160 = Address::from(log.topics[2]);

                let user_addr_hex =
                    "0x".to_string() + &hex::encode(user_addr_h160.to_fixed_bytes());
                let kol_addr_hex = "0x".to_string() + &hex::encode(kol_addr_h160.to_fixed_bytes());

                // 解析非索引数据
                let decoded_data = ethers::abi::decode(
                    &[
                        ethers::abi::ParamType::Uint(256),
                        ethers::abi::ParamType::Uint(256),
                    ],
                    &log.data,
                )
                .unwrap();

                let total_usd_x1000: u64 = decoded_data[0]
                    .clone()
                    .into_uint()
                    .unwrap_or_default()
                    .as_u64();

                let timestamp: u64 = decoded_data[1]
                    .clone()
                    .into_uint()
                    .unwrap_or_default()
                    .as_u64();

                // 用redis的有序集合来存储地址的积分流水:
                //   使用 address 作为 key
                //   使用 timestamp 作为 score , 这样可以按照时间戳排序
                //   使用 chainid_points_type_txhash 作为 member

                // 1:10给积分
                let user_rewards_points = total_usd_x1000 / 100;

                // TODO: 特殊kol,配置不同比例的积分比例不同,  默认按照user的10%给积分
                let mut kol_rewards_points = user_rewards_points * 10 / 100;
                if kol_addr_hex.eq("0x0000000000000000000000000000000000000000") {
                    // 0x0 是系统根账户，不奖励积分
                    kol_rewards_points = 0;
                }

                if user_rewards_points > 0 {
                    let user_member = chainid.to_string()
                        + "_"
                        + &(user_rewards_points.to_string())
                        + "_0_" // mint奖励
                        + &timestamp.to_string()
                        + "_"
                        + &log.transaction_hash.unwrap().encode_hex();
                    if kol_rewards_points > 0 {
                        let kol_member = chainid.to_string()
                            + "_"
                            + &(kol_rewards_points.to_string())
                            + "_1_" //邀请奖励
                            + &timestamp.to_string()
                            + "_"
                            + &log.transaction_hash.unwrap().encode_hex();

                        let _ = redis::pipe()
                            .zadd(
                                points_prefix.clone() + &user_addr_hex,
                                &user_member,
                                timestamp,
                            )
                            .zadd(
                                points_prefix.clone() + &kol_addr_hex,
                                &kol_member,
                                timestamp,
                            )
                            .query_async(&mut rds_conn)
                            .await?;
                    } else {
                        let _ = redis::pipe()
                            .zadd(
                                points_prefix.clone() + &user_addr_hex,
                                &user_member,
                                timestamp,
                            )
                            .query_async(&mut rds_conn)
                            .await?;
                    }
                }

                tracing::info!(
                    "MintNft Event: user_addr = {user_addr_hex}, to_addr = {kol_addr_hex}, total_usd_x1000= {total_usd_x1000}, timestamp={timestamp}"
                );
            }
        }

        // 更新数据库中扫描高度 + 1, 不重复扫描
        let new_from_block = tmp_to_block + 1;
        let _ = redis::pipe()
            .set(key_prefix.clone() + "cur_scan_block", new_from_block)
            .ignore()
            .query_async(&mut rds_conn)
            .await?;

        tracing::info!("ChainID-{chainid}更新扫描高度为{new_from_block}成功!");
        tmp_from_block = new_from_block;
    }
    Ok(())
}

fn gen_qrcode_text(chainid: u64, token_id: u64, token_id_owner: String) -> String {
    // 根据算法生成二维码
    let fansland_nft_contract_address = std::env::var(format!("FANSLAND_NFT_{}", chainid)).unwrap();
    let salt = "QrCode@fansland.io2024-888"; // TODO:
    let hash_msg = String::new()
        + &fansland_nft_contract_address
        + &token_id.to_string()
        + &token_id_owner
        + &chainid.to_string()
        + salt;
    let keccak_hash = ethers::utils::keccak256(hash_msg.as_bytes());
    let bz_qrcode = &keccak_hash[keccak_hash.len() - 15..];
    let qrcode = String::from("1:") + &hex::encode(bz_qrcode);

    return qrcode;
}
