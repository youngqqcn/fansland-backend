use ctrlc;
use dotenv::dotenv;
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
        // 更新积分排行榜
        let _ = update_points_rank().await;

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
    tracing::info!("更新积分排行榜");
    let rds_url = std::env::var("REDIS_URL").unwrap();
    tracing::debug!("rds_url: {}", rds_url);
    let rds_client = Client::open(rds_url).unwrap();
    let redis_pool = RedisPool::from(rds_client);
    let mut rds_conn = redis_pool.aquire().await?;

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

        // let mut historys: Vec<Point> = Vec::new();
        let mut total_points = 0_u64;
        if points_ret.len() > 0 && points_ret[0].len() > 0 {
            tracing::info!("points history length: {:?}", points_ret[0].len());
            for item in &points_ret[0] {
                let parts: Vec<&str> = item.split('_').collect();
                total_points += parts[1].parse::<u64>().unwrap();
            }
        }

        // 更新排行榜积分  ZADD key score1 member
        let address: &str = point_prefix_key.split(':').nth(1).unwrap_or("");
        let rank_prefix_key: String = String::from("pointsrank");
        let _: () = redis::pipe()
            .zadd(rank_prefix_key, &address.to_lowercase(), total_points)
            .query_async(&mut rds_conn)
            .await?;
    }

    Ok(())
}
