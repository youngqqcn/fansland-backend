use sha2::{Sha256, Digest};
fn main() {
    println!("Hello, world!");

    // for i in 10000.. {
    //     let sig_msg =
    //         format!("fansland_0xbfe5f435389ca190c3d3dec351db0ee9a8657a53_1705982410225_{i}"); // format!("abc{i}");

    //     let bz_sig = blake3::hash(sig_msg.as_bytes()).as_bytes().clone();
    //     if bz_sig[0] == 0 {
    //         // println!("{}", bz_sig.to_vec());
    //         bz_sig.map(|x| print!("{x},"));
    //         // println!("{}", hex::encode(bz_sig));
    //         println!("\n\n i = {}", i);
    //         break;
    //     }
    // }

    // 验证
    // let nonce = 1;
    // let sig_msg = format!("fansland_0x51bdbad59a24207b32237e5c47e866a32a8d5ed8_1705981597901_{nonce}"); // format!("abc{i}");

    // let bz_sig = blake3::hash(sig_msg.as_bytes()).as_bytes().clone();
    // if bz_sig[0] == 0 {
    //     bz_sig.map(|x| print!("{x},"));
    //     println!("{}", hex::encode(bz_sig));
    // } else {
    //     println!("verify failed")
    // }




    // let data = "fanslandweb3musicfestivalnftairdrop2024#001@test";
    // let data = "fanslandweb3musicfestivalnftairdrop2024#001@uat";
    let data = "fanslandweb3musicfestivalnftairdrop2024#001@pro";
    println!("{}", hex::encode( Sha256::digest(data)));

    // println!("SHA-256 Hash: {}", hex_string);
}
