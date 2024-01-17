# Fansland-backend
fansland后端


## 模块拆分


![](./docs/imgs/fansland-arch.png)

- `api`: 提供api接口供前端使用

- `nft`: 监听链上NFT合约`Transfer`事件

- `ticket`: 调用`Fantopia`API的接口生成门票二维码

- `email`: 发送邮件


## API

- 有鉴权
  - POST 绑定用户邮箱：/address/bindemail
  - GET 查询用户信息: /address/{address}
  - GET 查询用户门票列表：/address/tickets/{address}
  - POST 更新用户私密链接密码：updateslinkpasswd
- 无鉴权
  - POST 私密链接查询用户门票列表：/address/slink
  - GET 获取钱包登录签名消息：/siwe/msg/{address}
  - POST 钱包登录：/siwe/signin




## nft模块

- 监听NFT合约`MintNft`交易事件, 存入数据库
- 监听NFT合约`Transfer`交易事件, 存入数据库

## ticket模块

- 处理数据库中的Event,同时更新门票状态，调用Fantpia创建门票/刷新

## email模块

- 给用户发送门票邮件
