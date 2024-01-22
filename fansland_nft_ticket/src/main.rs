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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

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
        // 执行你的操作
        let _ = update_token_id_owner().await;
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
async fn update_token_id_owner() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = std::env::var("RPC_URL").unwrap();
    let provider = Provider::<Http>::try_from(rpc_url)?;

    abigen!(SimpleContract, "FanslandNFT.abi",);

    let fansland_nft_contract_address = std::env::var("FANSLAND_NFT").unwrap();
    let client = Arc::new(provider);
    // let contract_address: Address = CONTRACT_ADDRESS.parse()?;
    // let contract = SimpleContract::new(contract_address, client.clone());

    let rds_url = std::env::var("REDIS_URL").unwrap();
    tracing::debug!("rds_url: {}", rds_url);
    let rds_client = Client::open(rds_url).unwrap();
    let redis_pool = RedisPool::from(rds_client);
    let mut rds_conn = redis_pool.aquire().await?;

    // 获取当前数据库中的扫描起始高度
    let scan_from_block: u64 = match redis::cmd("GET")
        .arg("nft:scan:cur_block")
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
    tracing::info!("扫描区块范围:{scan_from_block} - {scan_to_block}");

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
        let prefix_key = format!("nft:tokenid:owner:{token_id}");
        let _ = redis::pipe()
            .set(prefix_key, &to_addr_hex)
            .sadd("nft:tokenids", token_id.as_u64())
            .sadd("nft:holders", &to_addr_hex)
            .ignore()
            .query_async(&mut rds_conn)
            .await?;
    }

    // 更新数据库中扫描高度
    let _ = redis::pipe()
        .set("nft:scan:cur_block", scan_to_block)
        .ignore()
        .query_async(&mut rds_conn)
        .await?;

    tracing::info!("更新扫描高度为{scan_to_block}成功");
    Ok(())
}

// 更新token的owner
// #[warn(dead_code)]
// async fn test() -> Result<(), Box<dyn std::error::Error>> {
//     let rpc_url = std::env::var("RPC_URL").unwrap();
//     let provider = Provider::<Http>::try_from(rpc_url)?;

//     abigen!(SimpleContract, "FanslandNFT.abi",);

//     let fansland_nft_contract_address = std::env::var("FANSLAND_NFT").unwrap();
//     let client = Arc::new(provider);
//     let contract_address: Address = fansland_nft_contract_address.parse()?;
//     let contract = SimpleContract::new(contract_address, client.clone());

//     println!("type of 0 : {}", contract.token_id_type_map(0.into()).await?);

//     Ok(())
// }
