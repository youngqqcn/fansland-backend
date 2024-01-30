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
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
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
    abigen!(SimpleContract, "FanslandNFT.abi",);

    let fansland_nft_contract_address = contract_address;
    let client = Arc::new(provider);

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
    let scan_to_block = latest_block.as_u64(); // 10 blocks to wait
    if scan_to_block <= scan_from_block {
        // 不用扫
        tracing::info!("已扫到最新区块:{scan_to_block}");
        return Ok(());
    }
    // TODO: 控制步长，步子不能太大，有些RPC不支持超过1000
    // if scan_to_block - scan_from_block > 100 {
    //     scan_to_block = scan_from_block + 100;
    // }
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

        // 插入数据库
        let _ = redis::pipe()
            .set(
                key_prefix.clone() + &format!("nft:tokenid:owner:{token_id}"),
                &to_addr_hex,
            )
            .sadd(key_prefix.clone() + "tokenids", token_id.as_u64())
            .sadd(key_prefix.clone() + "holders", &to_addr_hex)
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
    Ok(())
}
