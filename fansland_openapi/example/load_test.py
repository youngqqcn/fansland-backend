#coding:utf8

import hashlib
from pprint import pprint
import time
import requests


from locust import HttpUser, task, between

class QuickstartUser(HttpUser):
    # wait_time = between(1, 2)

    def on_start(self):
        # self.client.post("/login", json={"username":"foo", "password":"bar"})
        pass

    @task
    def get_qrcode(self):
        SALT = "ca17a3e225a85a74290831f504aceec5"
        # OPEN_URL = 'https://fansland.io/openapi-ok/v1/getQRCode'
        # OPEN_URL = 'http://127.0.0.1:3034/getQRCode'

        req = {
            "chain_id":56,
            "nft_contract":"0xBf36aB3AeD81Bf8553B52c61041904d98Ee882C2",
            "nft_owner":"0xe2bcf8373f6a6bd14189c7d4c5dbe7be8838937e",
            "nft_token_id":649,
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
        # print("签名消息: ", sig_msg)

        # 签名
        sig = hashlib.sha256(sig_msg.lower().encode()).hexdigest()
        print(f"签名结果: {sig}")

        # 设置签名
        req["signature"] = sig

        # req = {'chain_id': 56, 'nft_contract': '0xBf36aB3AeD81Bf8553B52c61041904d98Ee882C2', 'nft_owner': '0xe2bcf8373f6a6bd14189c7d4c5dbe7be8838937e', 'nft_token_id': 649, 'timestamp': 1711426868, 'signature': 'd11f7c65e2791372cce668c58583be9cbc11d05bbf6eb9c4ae0302f5a138aff6'}

        self.client.post("/getQRCode", json=req,headers={
            "Content-Type": "application/json",
            "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"
        })

    # @task(3)
    # def view_item(self):
    #     for item_id in range(10):
    #         self.client.get(f"/item?id={item_id}", name="/item")