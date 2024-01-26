#![allow(clippy::result_large_err)]

use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::{config::Region, Client, Error};
use dotenv::dotenv;
use email_address::EmailAddress;
use tracing::{info, warn, Level};

mod qrcode2b64;
mod template;

use crate::qrcode2b64::get_qrcode_png_base64;
use crate::template::get_email_template;

// Sends a message to all members of the contact list.
// snippet-start:[ses.rust.send-email]
async fn send_message(
    client: &Client,
    contact_list: Vec<String>,
    from_address: &str,
    message: &str,
) -> Result<(), Error> {
    println!("====================");

    let mut dest: Destination = Destination::builder().build();
    dest.to_addresses = Some(contact_list.clone());
    let subject_content = Content::builder()
        .data("fansland")
        .charset("UTF-8")
        .build()
        .expect("building Content");
    let body_content = Content::builder()
        .data(message)
        .charset("UTF-8")
        .build()
        .expect("building Content");
    let body = Body::builder().html(body_content).build();


    // 附件


    let msg = Message::builder()
        .subject(subject_content)
        .body(body)
        .build();

    let email_content = EmailContent::builder().simple(msg).build();

    client
        .send_email()
        .from_email_address(from_address)
        .destination(dest)
        .content(email_content)
        .send()
        .await?;

    tracing::info!("Email sent to {:?}", contact_list);

    Ok(())
}

// https://github.com/awslabs/aws-sdk-rust/blob/main/examples/examples/ses/src/bin/send-email.rs
#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // let message = "hello world";

    let rds_url = std::env::var("REDIS_URL").unwrap();
    tracing::debug!("rds_url: {}", rds_url);
    let rds_client = redis::Client::open(rds_url).unwrap();
    let redis_pool = redis_pool::RedisPool::from(rds_client);
    let mut rds_conn = redis_pool.aquire().await.unwrap();

    // 获取当前数据库中的扫描起始高度
    let send_to_address: String = match redis::cmd("LPOP")
        .arg("sendemail")
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
        .unwrap_or_default()
    {
        Some(m) => m,
        None => String::new(),
    };

    info!("address: {send_to_address}");

    if send_to_address != String::new() {
        let send_to_email: String = match redis::cmd("GET")
            .arg(format!("bindemail:{}", send_to_address.to_lowercase()))
            .query_async::<_, Option<String>>(&mut rds_conn)
            .await
            .unwrap_or_default()
        {
            Some(email_addr) => {
                if EmailAddress::is_valid(&email_addr) {
                    email_addr
                } else {
                    warn!("邮箱非法,不发送邮件==={email_addr}");
                    String::new()
                }
            }
            None => String::new(),
        };

        if send_to_email != String::new() {
            // 使用redis的 list 实现队列的FIFO:
            //   生产者：在fansland_nft_ticket中使用 rpush key value 在队列尾部(右边)插入值
            //   消费者： fansland_email使用 lpop key 从队列头部（左边）取值
            let qrcode_b64 = get_qrcode_png_base64("1:xxxxxxxxxxxxxxhhhhh");
            // 发送邮件
            let message = get_email_template(&send_to_address, &qrcode_b64);
            let from_address = "Fansland <no-reply@fansland.io>";
            let region = "ap-northeast-1";
            let shared_config = aws_config::from_env()
                .region(Region::new(region))
                .load()
                .await;
            let client = Client::new(&shared_config);
            send_message(&client, vec![send_to_email], from_address, &message).await?
        }
    }
    Ok(())
}
