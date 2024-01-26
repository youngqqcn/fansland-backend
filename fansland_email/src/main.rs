// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::result_large_err)]

use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::{config::Region, Client, Error};
use dotenv::dotenv;
use tracing::{debug, Level};

mod template;
use crate::template::get_zzzztemplate;


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
    dest.to_addresses = Some(contact_list);
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

    let msg = Message::builder()
        .subject(subject_content)
        .body(body)
        .build();

    let email_content = EmailContent::builder().simple(msg).build();
    // let email_content = EmailContent::builder().get_template()

    client
        .send_email()
        .from_email_address(from_address)
        .destination(dest)
        .content(email_content)
        .send()
        .await?;

    println!("Email sent to list");

    Ok(())
}

// https://github.com/awslabs/aws-sdk-rust/blob/main/examples/examples/ses/src/bin/send-email.rs
#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();

    // let message = "hello world";
    let message  = get_zzzztemplate("0x51Bdbad59a24207b32237e5c47E866A32a8D5Ed8");


    let receipent_emails = vec!["youngqqcn@gmail.com".to_string()];

    let rds_url = std::env::var("REDIS_URL").unwrap();
    tracing::debug!("rds_url: {}", rds_url);
    let rds_client = redis::Client::open(rds_url).unwrap();
    let redis_pool = redis_pool::RedisPool::from(rds_client);
    let mut rds_conn = redis_pool.aquire().await.unwrap();

    // 获取当前数据库中的扫描起始高度
    let send_to_email: String = match redis::cmd("LPOP")
        .arg("sendemail")
        .query_async::<_, Option<String>>(&mut rds_conn)
        .await
        .unwrap_or_default()
    {
        Some(m) => m,
        None => String::new(),
    };

    debug!("email: {send_to_email}");

    if send_to_email != String::new() {
        // 使用redis的 list 实现队列的FIFO:
        //   生产者：在fansland_nft_ticket中使用 rpush key value 在队列尾部(右边)插入值
        //   消费者： fansland_email使用 lpop key 从队列头部（左边）取值

        // 发送邮件
        let from_address = "Fansland <no-reply@fansland.io>";
        let region = "ap-northeast-1";
        let shared_config = aws_config::from_env()
            .region(Region::new(region))
            .load()
            .await;
        let client = Client::new(&shared_config);
        send_message(&client, receipent_emails, from_address, &message).await?
    }
    Ok(())
}
