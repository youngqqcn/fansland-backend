use chrono::{DateTime, Local, TimeZone, Utc};
use ctrlc;
use dotenv::dotenv;
use rand::Rng;
use redis::Client;
use redis_pool::RedisPool;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        // 更新积分排行榜
        let err = update_points_rank().await;
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
async fn update_points_rank() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("将EVM积分记录搞到mysql中,与任务积分合并");
    let rds_url = std::env::var("REDIS_URL").unwrap();
    tracing::debug!("rds_url: {}", rds_url);
    let rds_client = Client::open(rds_url).unwrap();
    let redis_pool = RedisPool::from(rds_client);
    let mut rds_conn = redis_pool.aquire().await?;
    let mut rng = rand::thread_rng();

    // mysql 数据库
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    // 使用redis
    let addresses_ret: Vec<Vec<String>> = redis::pipe()
        .keys("points:*")
        .query_async(&mut rds_conn)
        .await
        .map_err(|e| {
            tracing::error!("error: {:?}", e);
            e
        })?;

    tracing::info!("addresses: {:?}", addresses_ret.len());
    tracing::info!("addresses length: {:?}", addresses_ret[0].len());

    for point_prefix_key in &addresses_ret[0] {
        // 查询地址积分
        // 通过命令: ZREVRANGEBYSCORE points:0x51bdbad59a24207b32237e5c47e866a32a8d5ed8 9999999999 0 WITHSCORES
        tracing::info!("{point_prefix_key}");
        let points_ret: Vec<Vec<String>> = redis::pipe()
            .zrevrangebyscore(point_prefix_key, 9999999999_u64, 0)
            .query_async(&mut rds_conn)
            .await
            .map_err(|e| {
                tracing::error!("error: {:?}", e);
                e
            })?;

        tracing::info!("points history: {:?}", points_ret);
        let user_address: &str = point_prefix_key.split(':').nth(1).unwrap_or("");

        // let mut historys: Vec<Point> = Vec::new();
        // let mut total_points = 0_u64;
        if points_ret.len() > 0 && points_ret[0].len() > 0 {
            tracing::info!("points history length: {:?}", points_ret[0].len());
            for item in &points_ret[0] {
                let parts: Vec<&str> = item.split('_').collect();
                // total_points += parts[1].parse::<u64>().unwrap();
                let point_type = parts[2].parse::<u32>().unwrap();
                let tx_hash = parts[4];
                let chain_id = parts[0]; // 如果chainid过大，则需要修改数据库结构
                let points_amount = parts[1].parse::<u64>().unwrap();

                let ts = parts[3].parse::<u64>().unwrap();
                let date_time = Utc.timestamp_opt(ts as i64, 0).unwrap();
                let current_datetime: DateTime<Local> = Local::now();

                let cur_timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("获取时间戳失败")
                    .as_nanos()
                    + rng.gen_range(10000..=99999);

                let _ = sqlx::query!(
                    r#"
                        INSERT IGNORE INTO integral_request_record (id, app_id, request_type, hash, chain_id, address, create_time, update_time, amount )
                        VALUES (?, ?, ?, ?, ?, ?, ? ,?, ?)
                                "#,
                    &cur_timestamp.to_string()[0..18],
                    "evm_migrate",
                    point_type,
                    tx_hash,
                    chain_id,
                    user_address,
                    date_time.to_rfc3339(),
                    current_datetime.to_rfc3339(),
                    points_amount,
                )
                .execute(&pool)
                .await?
                .rows_affected();
            }
        }
    }

    Ok(())
}
