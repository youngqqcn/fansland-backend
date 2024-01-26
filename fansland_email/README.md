邮件使用CID发送图片:  https://juejin.cn/s/html%20inline%20image%20cid


https://docs.aws.amazon.com/ses/latest/dg/send-email-raw.html

```
cargo run -- --contact-list "youngqqcn@gmail.com" --region "ap-northeast-1" --from-address "Fansland <no-reply@fansland.io>" --message "hellowordskdfsdfdsf,this is a fansland test email,no reply please." --subject "helloworld" --verbose
```