// 安装依赖： npm install @noble/hashes

const blake3 = require("@noble/hashes/blake3").blake3;
// All params are optional

function makeRequestSig(address) {
  let timestamp = new Date().getTime();
  let msg = "fansland_" + address + "_" + timestamp + "_";
  msg = msg.toLocaleLowerCase(); // 转为小写

  console.log(msg);
  for (let nonce = 10000; nonce < 10000000; nonce++) {
    let sig_msg = msg + nonce;
    const hash = blake3(sig_msg, { dkLen: 32 });
    if (hash[0] == 0) {
      console.log(hash.toString());

      // 转为16进制字符串
      let hash_hex = Array.prototype.map
        .call(hash, function (byte) {
          return ("0" + (byte & 0xff).toString(16)).slice(-2);
        })
        .join("");
      console.log(hash_hex);

      return [timestamp, nonce];
    }
  }
  return [0, 0];
}

// 请求参数中的地址
req_address = "0x51bdbad59a24207b32237e5c47e866a32a8d5ed8";

let [timestamp, nonce] = makeRequestSig(req_address);

console.log("设置请求头里面的：");
console.log("FanslandTimestamp: ", timestamp);
console.log("FanslandNonce: ", nonce);

//
