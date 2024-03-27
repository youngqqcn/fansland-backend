use chrono::{DateTime, Local, TimeZone, Utc};
use clap::{arg, Parser};
use ctrlc;
use dotenv::dotenv;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use rand::Rng;
use redis::Client;
use redis_pool::RedisPool;
use std::env;
use std::num::{NonZeroU16, NonZeroUsize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let args = Args::parse();
    let chainid: u64 = args.chainid;
    let rpc_url = std::env::var(format!("RPC_URL_{chainid}")).unwrap();
    let contract_address = std::env::var(format!("FANSLAND_NFT_{chainid}")).unwrap();

    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
    //     )
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();
    // test().await?;

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
        // 更新积分排行榜
        let err = airdrop_task(&rpc_url, &contract_address, chainid).await;
        match err {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("error:{e}");
            }
        }

        // 睡眠一段时间，然后继续下一次循环
        // 1分钟更新一次即可
        for _ in 0..60 {
            sleep(Duration::from_secs(1)).await;
            if !r.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    println!("Received SIGKILL. Exiting...");

    Ok(())
}

// 更新积分排行榜
async fn airdrop_task(
    rpc_url: &String,
    contract_address: &String,
    chainid: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("web2用户兑换门票, 空投NFT");
    let rds_url = std::env::var("REDIS_URL").unwrap();
    tracing::debug!("rds_url: {}", rds_url);
    let rds_client = Client::open(rds_url).unwrap();
    let redis_pool = RedisPool::from(rds_client);
    let mut rds_conn = redis_pool.aquire().await?;
    // let mut rng = rand::thread_rng();

    let key = String::from("redeemairdrop:") + &chainid.to_string();

    let provider = Provider::<Http>::try_from(rpc_url)?;
    abigen!(FanslandNFTContract, "FanslandNFT.abi",);
    tracing::info!("nft_contract address: {}", contract_address);
    tracing::info!("rpc_url : {}", rpc_url);
    let client = Arc::new(provider);
    let fansland_nft_contract_address = contract_address;
    let contract_address_h160: Address = contract_address.parse()?;
    let contract = FanslandNFTContract::new(contract_address_h160, client.clone());


    // 使用redis
    let addresses_ret: Vec<Vec<String>> = redis::pipe()
        .lpop(key, Some(NonZeroUsize::new(1).unwrap()))
        .query_async(&mut rds_conn)
        .await
        .map_err(|e| {
            tracing::error!("error: {:?}", e);
            e
        })?;

    tracing::info!("addresses: {:?}", addresses_ret.len());
    tracing::info!("addresses length: {:?}", addresses_ret[0].len());


    // contract.redeem_airdrop(type_id, token_id, recipient);

    Ok(())
}
