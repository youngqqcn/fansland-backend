
## 有鉴权接口

- 绑定邮箱：`/bindEmail`

    ```shell
    curl --location '127.0.0.1:3000/bindEmail' \
    --header 'Content-Type: application/json' \
    --header 'FanslandAuthToken: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2FkZHJlc3MiOiIweGJmZTVmNDM1Mzg5Y2ExOTBjM2QzZGVjMzUxZGIwZWU5YTg2NTdhNTMiLCJleHAiOjE3MDU3MDg2NjV9.nW3fRCtu2S37nemL7b9LmdxUjL_bVkSWnbU3HX2tzbM' \
    --data-raw '{
        "address":"0xbfe5f435389ca190c3d3dec351db0ee9a8657a53",
        "email":"abcdxxxexxfa@gmail.com"
    }'

    ```


- 查询地址绑定的邮箱: `/queryAddressEmail`

    ```shell
    curl --location '127.0.0.1:3000/queryAddressEmail' \
    --header 'Content-Type: application/json' \
    --header 'FanslandAuthToken: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2FkZHJlc3MiOiIweGJmZTVmNDM1Mzg5Y2ExOTBjM2QzZGVjMzUxZGIwZWU5YTg2NTdhNTMiLCJleHAiOjE3MDU3MDg2NjV9.nW3fRCtu2S37nemL7b9LmdxUjL_bVkSWnbU3HX2tzbM' \
    --data '{
        "address":"0xbfe5f435389ca190c3d3dec351db0ee9a8657a53"
    }'
    ```


- 查询地址门票列表：`/queryTicketsByAddress`

    ```shell
    curl --location '127.0.0.1:3000/queryTicketsByAddress' \
    --header 'Content-Type: application/json' \
    --header 'FanslandAuthToken: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2FkZHJlc3MiOiIweGJmZTVmNDM1Mzg5Y2ExOTBjM2QzZGVjMzUxZGIwZWU5YTg2NTdhNTMiLCJleHAiOjE3MDU3MDg2NjV9.nW3fRCtu2S37nemL7b9LmdxUjL_bVkSWnbU3HX2tzbM' \
    --data '{
        "address": "0xbfe5f435389ca190c3d3dec351db0ee9a8657a53"
    }'
    ```


- 更新地址的私密链接密码：`/updateslinkpasswd`

    ```shell
    curl --location '127.0.0.1:3000/updateSlink' \
    --header 'Content-Type: application/json' \
    --header 'FanslandAuthToken: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2FkZHJlc3MiOiIweGJmZTVmNDM1Mzg5Y2ExOTBjM2QzZGVjMzUxZGIwZWU5YTg2NTdhNTMiLCJleHAiOjE3MDU3MDg2NjV9.nW3fRCtu2S37nemL7b9LmdxUjL_bVkSWnbU3HX2tzbM' \
    --data '{
        "address":"0xbfe5f435389ca190c3d3dec351db0ee9a8657a53",
        "passwd":"12345"
    }'
    ```

## 无鉴权接口

-  私密链接查询门票列表：`/slink`

    ```shell
    curl --location '127.0.0.1:3000/slink' \
    --header 'Content-Type: application/json' \
    --data '{
        "address":"0xbfe5f435389ca190c3d3dec351db0ee9a8657a53",
        "token":"7QKU2iRbEKrmc1yI8SG1",
        "passwd":"12345"
    }'
    ```

- 获取钱包登录签名消息：`/getSiweMsg`

    ```shell
    curl --location '127.0.0.1:3000/getSiweMsg' \
    --header 'Content-Type: application/json' \
    --data '{
        "address":"0xbfe5f435389ca190c3d3dec351db0ee9a8657a53",
        "chainid": 56
    }'
    ```

- 钱包登录：`/signInWithEthereum`

    ```shell
    curl --location '127.0.0.1:3000/signInWithEthereum' \
    --header 'Content-Type: application/json' \
    --data '{
        "address":"0xbfe5f435389ca190c3d3dec351db0ee9a8657a53",
        "msg":"localhost:8000 wants you to sign in with your Ethereum account:\n0xbfe5f435389ca190c3d3dec351db0ee9a8657a53\n\nWelcome to Fansland!\n\nURI: localhost:8000\nVersion: 1\nChain ID: 56\nNonce: 79692222\nIssued At: 2024-01-19T01:42:37.256Z",
        "sig":"0xe007d6cc5d3ec589b511ef628523832ccca4d1eb1642c33627590a0dc29a8f9f481820b1a908c2838d12cbc64913885c7f9c502998beaa1a5a6e118733f3dec51c"
    }'
    ```