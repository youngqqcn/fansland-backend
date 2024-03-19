import hashlib
from pprint import pprint
import time
import requests


def main():
    SALT = "ca17a3e225a85a74290831f504aceec5"
    OPEN_URL = 'https://fansland.io/openapi-ok/v1/getQRCode'
    # OPEN_URL = 'http://127.0.0.1:3034/getQRCode'

    req = {
        "chain_id":1,
        "nft_contract":"0xE9AE3261a475a27Bb1028f140bc2a7c843318afD",
        "nft_owner":"0xF4435c244A292a8E8D56767bf6DF9b9c4D59aEED",
        "nft_token_id":66,
        "timestamp": int(time.time()),
        "signature":""
    }

    # 组成签名消息
    sig_msg = (
        str(req['chain_id'])
        + "&"
        + req['nft_contract']
        + "&"
        + req['nft_owner']
        + "&"
        + str(req['nft_token_id'])
        + "&"
        + str(req['timestamp'])
        + "&"
        + SALT
    )
    print("签名消息: ", sig_msg)

    # 签名
    sig = hashlib.sha256(sig_msg.lower().encode()).hexdigest()
    print(f"签名结果: {sig}")

    # 设置签名
    req["signature"] = sig


    # 请求接口
    resp = requests.post(
        url=OPEN_URL,
        json=req,
        headers={
        "Content-Type": "application/json",
        "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"
    },)
    print(resp.text)

    # 响应结果
    pprint(resp.json())



if __name__ == '__main__':
    main()
