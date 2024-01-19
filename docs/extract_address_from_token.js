/**
 * 从私密链接的token里面提取地址
 */


let token = "QU1ZaTNqN2FMer_l9DU4nKGQw9Pew1HbDumoZXpT"; // 示例 token
token = token.replace(/-/g, "+").replace(/_/g, "/"); // 处理 Base64 URL-Safe 编码的输入

let bz_token = Array.from(atob(token), (c) => c.charCodeAt(0)); // 将 base64 解码为字节数组
let raw_token = String.fromCharCode(...bz_token.slice(0, 10)); // 提取原始token字节数组
let raw_address = Array.from(bz_token.slice(10)); // 提取地址字节数组

let address =
  "0x" + raw_address.map((byte) => byte.toString(16).padStart(2, "0")).join(""); // 将字节数组转换为十六进制字符串

console.log("解码后的地址:", address);
