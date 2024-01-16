use ethers::types::{Address, Signature};
use std::str::FromStr;

// https://github.com/MetaMask/test-dapp/blob/0ea6cc7a496eb0049c735b4deffcc0ba9b234281/src/index.js#L1849C1-L1857C5
// 签名代码: https://github.com/MetaMask/test-dapp/blob/0ea6cc7a496eb0049c735b4deffcc0ba9b234281/src/index.js#L1834C2-L1847C5

// 消息模板： `${domain} wants you to sign in with your Ethereum account:\n${from}\n\nWelcome to Fansland! This request will NOT trigger a blockchain transaction or cost any gas fees.\n\nURI: https://${domain}\nVersion: 1\nChain ID: 1\nNonce: 32891757\nIssued At: 2021-09-30T16:25:24.000Z`;

/*
Welcome to OpenSea!

Click to sign in and accept the OpenSea Terms of Service (https://opensea.io/tos) and Privacy Policy (https://opensea.io/privacy).

This request will not trigger a blockchain transaction or cost any gas fees.

Wallet address:
0x624c87ab2ccb5cb8fa3054984a9b3f6b97017751

Nonce:
347efdd5-10fd-4f59-b4a0-59ea36fc1624
*/

// fn verify_signature(
//     msg: String,
//     sig: String,
//     address: String,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // 将字符串签名转换为 Signature 类型
//     let signature = Signature::from_str(&sig)?;
//     let address = Address::from_str(&address)?;

//     signature.verify(msg.clone(), address)?;

//     let addr = signature.recover(msg.clone())?;
//     println!("addr: {}", addr.to_string());
//     if addr == address {
//         println!("verify ok");
//     } else {
//         println!("verify failed");
//     }
//     Ok(())
// }

pub fn verify_signature(msg: String, sig: String, address: String) -> bool {
    match (Signature::from_str(&sig), Address::from_str(&address)) {
        (Ok(signature), Ok(address)) => {
            if signature.verify(msg.clone(), address).is_ok() {
                let recovered_address = signature.recover(msg.as_bytes()).ok();
                if let Some(addr) = recovered_address {
                    println!("addr: {}", addr.to_string());
                    return addr == address;
                }
            }
        }
        _ => {}
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let msg = "6c6f63616c686f73743a393031312077616e747320796f7520746f207369676e20696e207769746820796f757220457468657265756d206163636f756e743a0a3078626665356634333533383963613139306333643364656333353164623065653961383635376135330a0a492061636365707420746865204d6574614d61736b205465726d73206f6620536572766963653a2068747470733a2f2f636f6d6d756e6974792e6d6574616d61736b2e696f2f746f730a0a5552493a2068747470733a2f2f6c6f63616c686f73743a393031310a56657273696f6e3a20310a436861696e2049443a20310a4e6f6e63653a2033323839313735370a4973737565642041743a20323032312d30392d33305431363a32353a32342e3030305a";
        // let decoded_bytes = Vec::from_hex(hex_string).expect("Failed to decode hex string");

        // 将字节数组转换为字符串
        let d = hex::decode(msg).unwrap();
        let raw_msg = String::from_utf8(d).unwrap();
        println!("raw_msg:{}", raw_msg);

        let x = verify_signature(raw_msg,
        "0x92ef936ef3470d6564d34c169ae1392523a11df9e71fa799df2c66c16dc4c33b11b1bd1dc84db7448f97962dc8574f02b656743c54455a796a97c64d4ce56d7b1b".to_owned(),
        "0xbfe5f435389ca190c3d3dec351db0ee9a8657a53".to_owned());
        println!("verify: {}", x);
        assert!(x == true);
    }

    #[test]
    fn test_failed() {
        let msg = "116c6f63616c686f73743a393031312077616e747320796f7520746f207369676e20696e207769746820796f757220457468657265756d206163636f756e743a0a3078626665356634333533383963613139306333643364656333353164623065653961383635376135330a0a492061636365707420746865204d6574614d61736b205465726d73206f6620536572766963653a2068747470733a2f2f636f6d6d756e6974792e6d6574616d61736b2e696f2f746f730a0a5552493a2068747470733a2f2f6c6f63616c686f73743a393031310a56657273696f6e3a20310a436861696e2049443a20310a4e6f6e63653a2033323839313735370a4973737565642041743a20323032312d30392d33305431363a32353a32342e3030305a";

        let d = hex::decode(msg).unwrap();
        // println!("hex_string:{}", hex_string);
        let raw_msg = String::from_utf8(d).unwrap();
        println!("raw_msg:{}", raw_msg);

        let x = verify_signature(raw_msg,
        "0x92ef936ef3470d6564d34c169ae1392523a11df9e71fa799df2c66c16dc4c33b11b1bd1dc84db7448f97962dc8574f02b656743c54455a796a97c64d4ce56d7b1b".to_owned(),
        "0xbfe5f435389ca190c3d3dec351db0ee9a8657a53".to_owned());
        println!("verify: {}", x);
        assert!(x == false, "xx");
    }
}
