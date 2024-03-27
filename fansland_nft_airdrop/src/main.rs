use clap::{arg, Parser};
use ctrlc;
use dotenv::dotenv;
use redis::Client;
use redis_pool::RedisPool;
use std::io::{Error, ErrorKind};
use std::num::NonZeroUsize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ethers::{
    contract::abigen,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
// use eyre::Result;
use std::convert::TryFrom;
use tracing::Level;

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

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

    let key = String::from("redeemairdrop:") + &chainid.to_string();

    // 使用redis
    let values: Vec<Vec<String>> = redis::pipe()
        .lpop(key.clone(), Some(NonZeroUsize::new(1).unwrap()))
        .query_async(&mut rds_conn)
        .await
        .map_err(|e| {
            tracing::error!("error: {:?}", e);
            e
        })?;

    tracing::info!("addresses: {:?}", values.len());
    tracing::info!("addresses length: {:?}", values[0].len());

    if values[0].len() == 0 {
        tracing::info!("暂时没有空投消息");
        return Ok(());
    }

    tracing::info!("values[0]: {:?}", &values[0][0]);

    // 按照下划线分割
    let raw_airdrop_msg_str = values[0][0].clone();
    let airdrop_msgs: Vec<&str> = raw_airdrop_msg_str
        .split('_')
        .filter(|&x| x != "airdrop")
        .collect();
    tracing::info!("airdrop_msgs: {:?}", airdrop_msgs);
    if airdrop_msgs.len() != 4 {
        return Err(Box::new(Error::new(ErrorKind::Other, "空投消息非法")));
    }

    let type_id = airdrop_msgs[0].parse::<u64>()?;
    let chain_id = airdrop_msgs[1].parse::<u64>()?;
    let token_id = airdrop_msgs[2].parse::<u64>()?;
    let recipient = airdrop_msgs[3].to_owned();

    // 检查tokenId是否已经上链
    let check_key = format!("nft:{0}:nft:tokenid:owner:{1}", chain_id, token_id);
    match redis::cmd("GET")
        .arg(&check_key)
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
    {
        Ok(x) => match x {
            Some(k) => {
                tracing::error!(
                    "chainId:{}, tokenId:{}已经mint,owner:{}, 不能重复mint。",
                    chain_id,
                    token_id,
                    k
                );

                // 这里直接返回
                return Ok(());
            }
            None => {
                tracing::info!("chainId:{}, tokenId:{}没有mint", chain_id, token_id);
            }
        },
        Err(e) => {
            // 将已经取出的消息，插回去, 不一定能成功
            // TODO: 优化这种处理方式
            tracing::info!(
                "redis发生错误,将消息写回redis, key: {}  , value: {}",
                &key,
                &raw_airdrop_msg_str
            );
            let _ = redis::pipe()
                .rpush(key.clone(), raw_airdrop_msg_str.clone())
                .query_async(&mut rds_conn)
                .await
                .map_err(|e| {
                    tracing::error!("error: {:?}", e);
                    e
                })?;
            return Err(Box::new(e));
        }
    }

    // TODO: 检查是否存在消息处理记录， 防止短时间内的重复消息
    //

    // 检查 recipient 是否合法
    let _ = recipient.parse::<Address>()?;

    match airdrop(
        chain_id,
        rpc_url.clone(),
        contract_address.clone(),
        type_id,
        recipient,
        token_id,
    )
    .await
    {
        Ok(_) => {
            // TODO: 将txhash插入redis, 作为记录
        }
        Err(e) => {
            // 发生错误，将key和redis重新写回redis中，防止消息丢失
            tracing::error!("airdrop发生错误: {}", e);
            tracing::info!(
                "将消息写回redis, key: {}  , value: {}",
                &key,
                &raw_airdrop_msg_str
            );
            let _ = redis::pipe()
                .rpush(key.clone(), raw_airdrop_msg_str.clone())
                .query_async(&mut rds_conn)
                .await
                .map_err(|e| {
                    tracing::error!("error: {:?}", e);
                    e
                })?;
        }
    }

    Ok(())
}

#[warn(dead_code)]
async fn airdrop(
    chain_id: u64,
    rpc_url: String,
    contract_address: String,
    type_id: u64,
    recipient: String,
    token_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "开始空投===chainId:{}, contract:{}, type_id:{}, recipient:{}, token_id:{}",
        chain_id,
        contract_address,
        type_id,
        recipient,
        token_id
    );

    abigen!(FanslandNFTContract, "FanslandNFT.abi",);
    tracing::info!("nft_contract address: {}", &contract_address);
    tracing::info!("rpc_url : {}", rpc_url);
    let contract_address: Address = contract_address.parse()?;
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // TODO: fix private key
    let from_wallet: LocalWallet =
        "0xa1102aa1ecf406a2633bd227efc4ecd16aa5c642d3b85a606b7b20fad109a50d"
            .parse::<LocalWallet>()?
            .with_chain_id(chain_id);

    let signer = Arc::new(SignerMiddleware::new(provider, from_wallet));
    tracing::info!("==================");
    tracing::info!("from_wallet: {}", signer.address());
    let contract = FanslandNFTContract::new(contract_address, signer);

    let type_id: U256 = type_id.into();
    let token_id: U256 = token_id.into();
    let recipient: Address = recipient.parse()?;

    let tx = contract.redeem_airdrop(type_id, token_id, recipient);
    tracing::info!("raw_tx: {:?}", tx);
    tracing::info!("==================");

    let pending_tx = tx.send().await?;
    tracing::info!("pending_tx: {:?}", pending_tx);
    tracing::info!("==================");

    let mined_tx = pending_tx.await?;
    tracing::info!("minted_tx: {:?}", mined_tx);
    tracing::info!("==================");

    Ok(())
}
