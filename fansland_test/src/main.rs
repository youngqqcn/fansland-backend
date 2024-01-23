// use reqwest::blocking::get;

use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use super::*;
    use ethers::{
        core::k256::ecdsa::SigningKey,
        signers::{Signer, Wallet, WalletError},
        types::{Address, Signature, SignatureError},
    };
    use reqwest::{Client, Error};

    const JWT_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2FkZHJlc3MiOiIweGNkOWNmZjZhOTVkNmJiN2Q4YjhiNTBlYzg5NjUxM2U5YTJjZjY1NGEiLCJleHAiOjE3MDYwODYwNzh9.kLS1tb6wz7maiF7UgFFhlArtIu0zALqroTiUYwHFFzc";

    fn make_sig(address: &str) -> (u128, i32) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        for i in 10000.. {
            let sig_msg = format!("fansland_{address}_{timestamp}_{i}"); // format!("abc{i}");
            let bz_sig = blake3::hash(sig_msg.as_bytes()).as_bytes().clone();
            if bz_sig[0] == 0 {
                // println!("{}", bz_sig.to_vec());
                bz_sig.map(|x| print!("{x},"));
                // println!("{}", hex::encode(bz_sig));
                println!("\n\n i = {}", i);
                return (timestamp, i);
            }
        }
        return (0, 0);
    }

    pub fn verify_signature(
        msg: String,
        sig: String,
        address: String,
    ) -> Result<(), SignatureError> {
        // 将字符串签名转换为 Signature 类型
        let signature = Signature::from_str(&sig)?;
        let address = Address::from_str(&address).unwrap();

        signature.verify(msg.clone(), address)
    }

    pub async fn make_siwe(
        msg: String,
        // address: String,
    ) -> Result<String, WalletError> {
        let address = "0xCD9cFf6a95d6bb7d8b8B50eC896513E9a2CF654a";
        let s = "fc380c443c077f2b76d9ed5472f6de7fcd383c9975123300921c1572a5122482";
        let wallet: Wallet<SigningKey> = s.parse().unwrap();
        let sig = wallet.sign_message(msg.as_bytes()).await?;
        println!("====");
        println!("{msg}");
        println!("====");

        if let Ok(_) = verify_signature(msg, sig.to_string(), address.to_owned()) {
            println!("verify ok")
        } else {
            println!("failed")
        }

        Ok(sig.to_string())
    }

    #[tokio::test]
    async fn test_make_siwe_sig() {
        let sig = make_siwe("hello".to_owned()).await.unwrap();
        println!("sig: {sig}")
    }

    // pub struct SiweRsp {}

    #[tokio::test]
    async fn test_get_siwe_msg() -> Result<(), Error> {
        // 创建 reqwest 的 Client 实例
        let client = Client::new();

        // 准备要发送的 JSON 数据
        let address = "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a".to_lowercase();
        let (ts, nonce) = make_sig(&address);

        // 发送 HTTP POST 请求
        let res = client
            .post("http://127.0.0.1:3000/getSiweMsg")
            .json(&serde_json::json!({
                "address": address,
                "chainid": 56
            }))
            .header("Content-Type", "application/json")
            .header("Fansland-Token", "--")
            .header("Fansland-Timestamp", format!("{ts}"))
            .header("Fansland-Nonce", format!("{nonce}"))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        println!("111 res: {res}");

        if let Some(signmsg) = res.get("data").and_then(|data| data.get("signmsg")) {
            println!("==signmsg==");
            println!("{signmsg}");
            println!("====");

            // TODO: json序列化的时候有问题, 暂时通过字符串替换来还原
            let new_signmsg = signmsg.to_string().replace("\\n", "\n").replace("\"", "");

            println!("==new_signmsg==");
            println!("{new_signmsg}");
            println!("====");

            let sig = make_siwe(new_signmsg.clone()).await.unwrap();
            println!("====sig = {}", sig);

            // 发送 HTTP POST 请求
            let rsp = client
                .post("http://127.0.0.1:3000/signInWithEthereum")
                .json(&serde_json::json!({
                    "address": "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a",
                    "msg":new_signmsg.clone(),
                    "sig":sig,
                }))
                .header("Content-Type", "application/json")
                .header("Fansland-Token", "--")
                .header("Fansland-Timestamp", format!("{ts}"))
                .header("Fansland-Nonce", format!("{nonce}"))
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            println!("2222 {rsp}");
            if let Some(code) = rsp.get("code") {
                if code == 0 {
                    println!("ok");
                }
            } else {
                assert!(false, "get code failed");
            }
        } else {
            // println!("failed")
            assert!(false, "get signmsg failed");
        }

        // println!("res: {}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_qr_code_by_slink() -> Result<(), Error> {
        // 创建 reqwest 的 Client 实例
        let client = Client::new();

        // 准备要发送的 JSON 数据
        let address = "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a".to_lowercase();
        let (ts, nonce) = make_sig(&address);

        // 发送 HTTP POST 请求
        let res = client
            .post("http://127.0.0.1:3000/queryQrCodeBySlink")
            .json(&serde_json::json!({
                "address": "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a",
                "chainid": 56,
                "token_id":17,
                "token": "dVVCa1BMS1Jsd82c_2qV1rt9i4tQ7IllE-miz2VK",
                "passwd":"13345"
            }))
            .header("Content-Type", "application/json")
            // .header("Fansland-Token", JWT_TOKEN)
            .header("Fansland-Timestamp", format!("{ts}"))
            .header("Fansland-Nonce", format!("{nonce}"))
            .send()
            .await?
            .text()
            .await?;

        println!("res: {}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_sign_in_with_ethereum() -> Result<(), Error> {
        // 创建 reqwest 的 Client 实例
        let client = Client::new();

        // 准备要发送的 JSON 数据
        let address = "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a".to_lowercase();
        let (ts, nonce) = make_sig(&address);

        // 发送 HTTP POST 请求
        let res = client
            .post("http://127.0.0.1:3000/signInWithEthereum")
            .json(&serde_json::json!({
                "address": "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a",
                "msg":"",
                "sig":"",
            }))
            .header("Content-Type", "application/json")
            .header("Fansland-Token", JWT_TOKEN)
            .header("Fansland-Timestamp", format!("{ts}"))
            .header("Fansland-Nonce", format!("{nonce}"))
            .send()
            .await?
            .text()
            .await?;

        println!("res: {}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_bind_email() -> Result<(), Error> {
        // 创建 reqwest 的 Client 实例
        let client = Client::new();

        // 准备要发送的 JSON 数据
        let address = "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a".to_lowercase();
        let (ts, nonce) = make_sig(&address);

        // 发送 HTTP POST 请求
        let res = client
            .post("http://127.0.0.1:3000/bindEmail")
            .json(&serde_json::json!({
                "address": "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a",
                "email":"youngqqcn@gmail.com",
            }))
            .header("Content-Type", "application/json")
            .header("Fansland-Token", JWT_TOKEN)
            .header("Fansland-Timestamp", format!("{ts}"))
            .header("Fansland-Nonce", format!("{nonce}"))
            .send()
            .await?
            .text()
            .await?;

        println!("res: {}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_address_email() -> Result<(), Error> {
        // 创建 reqwest 的 Client 实例
        let client = Client::new();

        // 准备要发送的 JSON 数据
        let address = "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a".to_lowercase();
        let (ts, nonce) = make_sig(&address);

        // 发送 HTTP POST 请求
        let res = client
            .post("http://127.0.0.1:3000/queryAddressEmail")
            .json(&serde_json::json!({
                "address": "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a",
            }))
            .header("Content-Type", "application/json")
            .header("Fansland-Token", JWT_TOKEN)
            .header("Fansland-Timestamp", format!("{ts}"))
            .header("Fansland-Nonce", format!("{nonce}"))
            .send()
            .await?
            .text()
            .await?;

        println!("res: {}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_qr_code_by_address() -> Result<(), Error> {
        // 创建 reqwest 的 Client 实例
        let client = Client::new();

        // 准备要发送的 JSON 数据
        let address = "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a".to_lowercase();
        let (ts, nonce) = make_sig(&address);

        // 发送 HTTP POST 请求
        let res = client
            .post("http://127.0.0.1:3000/queryQrCodeByAddress")
            .json(&serde_json::json!({
                "address": "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a",
                "token_id": 17,
            }))
            .header("Content-Type", "application/json")
            .header("Fansland-Token", JWT_TOKEN)
            .header("Fansland-Timestamp", format!("{ts}"))
            .header("Fansland-Nonce", format!("{nonce}"))
            .send()
            .await?
            .text()
            .await?;

        println!("res: {}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_slink() -> Result<(), Error> {
        // 创建 reqwest 的 Client 实例
        let client = Client::new();

        // 准备要发送的 JSON 数据
        let address = "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a".to_lowercase();
        let (ts, nonce) = make_sig(&address);

        // 发送 HTTP POST 请求
        let res = client
            .post("http://127.0.0.1:3000/updateSlink")
            .json(&serde_json::json!({
                "address": "0xcd9cff6a95d6bb7d8b8b50ec896513e9a2cf654a",
                "passwd": "13345",
            }))
            .header("Content-Type", "application/json")
            .header("Fansland-Token", JWT_TOKEN)
            .header("Fansland-Timestamp", format!("{ts}"))
            .header("Fansland-Nonce", format!("{nonce}"))
            .send()
            .await?
            .text()
            .await?;

        println!("res: {}", res);
        Ok(())
    }
}
