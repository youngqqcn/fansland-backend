use clap::{arg, Parser};
use ctrlc;
use dotenv::dotenv;
use ethers::providers::PendingTransaction;
use hex::ToHex;
use redis::Client;
use redis_pool::RedisPool;
use sha2::{Digest, Sha256};
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

    /// env , test, uat, pro
    #[arg(short, long)]
    env: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let args = Args::parse();
    let chainid: u64 = args.chainid;
    let rpc_url = std::env::var(format!("RPC_URL_{chainid}")).unwrap();
    let contract_address = std::env::var(format!("FANSLAND_NFT_{chainid}")).unwrap();
    let pk = match args.env.as_str() {
        // 6c696ad29d0fbc55aca9ebc07395406e3c396d6ae4684182b99f60f23f7d9b36
        // 0xDEe74737Aa7C9E75cc782419D97DE18Eb2918e81
        "test" => hex::encode(Sha256::digest(
            "fanslandweb3musicfestivalnftairdrop2024#001@test",
        )),
        // 2a6bb518db5e8e643b2c8ef472a0e958111687a1c48561426bfb604197292ab6
        // 0x8d39F5882F1F49714612FF06328189aAc9915728
        "uat" => hex::encode(Sha256::digest(
            "fanslandweb3musicfestivalnftairdrop2024#001@uat",
        )),
        // bd1cd9fbe6705932f7252a4ccd177c1d59bd76c61973349621d780d9edf1a19d
        // 0x7691cd47462D7659e69DAD7561878e2A31b41cfB
        "pro" => hex::encode(Sha256::digest(
            "fanslandweb3musicfestivalnftairdrop2024#001@pro",
        )),
        _ => panic!("无效chainid"),
    };

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
        let err = airdrop_task(&rpc_url, &contract_address, chainid, pk.clone()).await;
        match err {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("error:{e}");
            }
        }

        // 睡眠一段时间，然后继续下一次循环
        // 1分钟更新一次即可
        for _ in 0..3 {
            sleep(Duration::from_secs(1)).await;
            if !r.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    tracing::info!("Received SIGKILL. Exiting...");

    Ok(())
}

// 更新积分排行榜
async fn airdrop_task(
    rpc_url: &String,
    contract_address: &String,
    chainid: u64,
    pk: String,
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
    let raw_airdrop_msg_str = values[0][0].clone().to_lowercase();

    // 按照下划线分割
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
    // 检查 recipient 是否合法
    let _ = recipient.parse::<Address>()?;

    // 检查tokenId是否已经上链
    let check_key = format!("nft:{0}:nft:tokenid:owner:{1}", chain_id, token_id);
    match redis::cmd("GET")
        .arg(&check_key)
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await?
    {
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
    }

    // 检查是否存在消息处理记录， 防止短时间内的重复消息
    let msg_handle_record_key = format!("airdroplogs:{}:{}", chain_id, raw_airdrop_msg_str);
    match redis::cmd("GET")
        .arg(&msg_handle_record_key)
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await?
    {
        Some(v) => {
            if v == "peding" {
                tracing::info!("旧消息,之前处理过,但是没有完全成功, 继续处理");
            } else if v.starts_with("0x") && v.len() == 66 {
                tracing::info!(
                    "此消息已处理且空投成功, 不再重复处理, 消息: {},  txhash: {}",
                    raw_airdrop_msg_str,
                    v
                );

                return Ok(());
            }
        }
        None => {
            tracing::info!("新消息，但是没有最终成功, 继续处理");
        }
    }

    // 如果有空投消息, 先将空投消息重新写回去（写到队尾），防止后面处理过程中出错,导致消息丢失
    let _ = redis::pipe()
        .rpush(key.clone(), raw_airdrop_msg_str.clone())
        .query_async(&mut rds_conn)
        .await
        .map_err(|e| {
            tracing::error!("消息冗余写回失败: {:?}", e);
            e
        })?;
    tracing::info!(
        "消息冗余写回成功, key: {}  , value: {}",
        &key,
        &raw_airdrop_msg_str
    );

    // 插入消息处理记录, value默认是 pending, 表示已经开始处理, 但未成功
    let _ = redis::pipe()
        .set(msg_handle_record_key.clone(), "pending")
        .query_async(&mut rds_conn)
        .await
        .map_err(|e| {
            tracing::error!("error: {:?}", e);
            e
        })?;

    // 开始空投
    match airdrop(
        chain_id,
        rpc_url.clone(),
        contract_address.clone(),
        type_id,
        recipient,
        token_id,
        pk,
    )
    .await
    {
        Ok(txhash) => {
            // 将txhash插入redis, 作为记录
            tracing::info!("交易成功, 将txhash写入redis保存, txhash:{}", txhash);

            let _ = redis::pipe()
                .set(msg_handle_record_key.clone(), txhash.clone())
                .query_async(&mut rds_conn)
                .await
                .map_err(|e| {
                    tracing::error!("error: {:?}", e);
                    e
                })?;
        }
        Err(e) => {
            // 因为一开始就已经冗余写回, 因此这里不做写回操作
            return Err(e);
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
    pk: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let pk = match chain_id {
        80001 => "0xa1102aa1ecf406a2633bd227efc4ecd16aa5c642d3b85a606b7b20fad109a50d",
        137 => "fanslandweb3musicfestivalnftairdrop2024#001@uat",
        _ => {
            tracing::info!("=====无效chain_id=====: {}", chain_id);
            return Err(Box::new(std::io::Error::new(
                ErrorKind::Other,
                "无效chain_id",
            )));
        }
    };

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
    let from_wallet: LocalWallet = pk.parse::<LocalWallet>()?.with_chain_id(chain_id);

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

    let pending_tx: PendingTransaction<Http> = tx.send().await?;
    let txhash: String = "0x".to_owned() + &pending_tx.tx_hash().encode_hex::<String>();
    tracing::info!("交易已广播,等待区块确认: {:?}", pending_tx);
    tracing::info!("==================");

    let mined_tx = pending_tx.await?;
    tracing::info!("交易已确认,交易回执: {:?}", mined_tx);
    tracing::info!("==================");
    if let Some(x) = mined_tx {
        if let Some(status) = x.status {
            if status == 1.into() {
                // 成功
                tracing::info!("=====交易成功=====:{}", txhash);
            } else {
                // 失败
                tracing::info!("=====交易失败=====txhash: {}", txhash);
                return Err(Box::new(std::io::Error::new(ErrorKind::Other, "交易失败")));
            }
        }
    }
    Ok(txhash)
}
