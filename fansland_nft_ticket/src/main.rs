use clap::Parser;
use ctrlc;
use dotenv::dotenv;
use ethers::{
    contract::abigen,
    core::types::{Address, Filter, U256},
    providers::{Http, Middleware, Provider},
};
use redis::Client;
use redis_pool::RedisPool;
use std::time::Duration;
use std::{
    f64::consts::E,
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
        let _ = update_token_id_owner(&rpc_url, &contract_address, chainid).await;
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

    let key_prefix = "nft:".to_owned() + &chainid.to_string() + ":";

    // 获取当前数据库中的扫描起始高度
    let mut scan_from_block: u64 = match redis::cmd("GET")
        .arg(key_prefix.clone() + "cur_scan_block")
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await?
    {
        Some(m) => m.parse()?,
        None => 0,
    };

    // 获取当前最新高度
    let latest_block = client.get_block_number().await?;
    let mut scan_to_block = latest_block.as_u64(); // 10 blocks to wait
    if scan_to_block <= scan_from_block {
        // 不用扫
        tracing::info!("已扫到最新区块:{scan_to_block}");
        return Ok(());
    }
    // TODO: 控制步长，步子不能太大，有些RPC不支持超过1000
    let mut continue_loop = true;
    while continue_loop {
        if scan_to_block - scan_from_block > 500 {
            scan_to_block = scan_from_block + 500;
            continue_loop = true;
        } else {
            continue_loop = false;
        }
        if scan_from_block == 0 {
            scan_from_block = scan_to_block - 100;
        }
        tracing::info!("ChainID-{chainid}扫描区块范围:{scan_from_block} - {scan_to_block}");

        // 合约的转移事件： event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
        let filter = Filter::new()
            .address(fansland_nft_contract_address.parse::<Address>()?)
            .event("Transfer(address,address,uint256)")
            .from_block(scan_from_block)
            .to_block(scan_to_block);

        let logs = client.get_logs(&filter).await?;
        tracing::info!("{} Transfer events found!", logs.iter().len());
        for log in logs.iter() {
            let from_addr_h160 = Address::from(log.topics[1]);
            let to_addr_h160 = Address::from(log.topics[2]);

            let from_addr_hex = "0x".to_string() + &hex::encode(from_addr_h160.to_fixed_bytes());
            let to_addr_hex = "0x".to_string() + &hex::encode(to_addr_h160.to_fixed_bytes());

            let token_id = U256::from_big_endian(&log.topics[3].as_bytes()[29..32]);
            tracing::info!(
                "from_addr = {from_addr_hex}, to_addr = {to_addr_hex}, token_id= {token_id}"
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
                .ignore()
                .query_async(&mut rds_conn)
                .await?;
        }

        // 更新数据库中扫描高度
        let _ = redis::pipe()
            .set(key_prefix.clone() + "cur_scan_block", scan_to_block)
            .ignore()
            .query_async(&mut rds_conn)
            .await?;

        tracing::info!("ChainID-{chainid}更新扫描高度为{scan_to_block}成功!");
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
