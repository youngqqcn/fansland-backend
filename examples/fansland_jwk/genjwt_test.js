const jwt = require("jsonwebtoken");
const uuid  = require('uuid');

const privateKey = `-----BEGIN PRIVATE KEY-----
MIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQDXOiJsKvsB5t0a
mR55J6tR9aZGodZJi8Bypo7/JGAvblTYSP8tFbdfqSgFCLsZhU0bLn8kQNQFlfr7
P2IarEeAeuqfa8xrh1ys9QAdtzoZv5TfDxj2Xc26CRuS78eQu+cNvP8JPajetkid
tMUM5+FPvsU1f6PBqFEHFdGV6i3RRK16VuCvBhzDlpEmGpKIxEl90w7WYW6zUqwC
+i1uMSo1MrZkxNnTRzDzk2ryu4N5qq6M18yZ9VmH5mPdshTbhFxTq9bKDl3W1Rc9
cMaOu1261n88YF8ZljH/91zwRM0nimmHFDVkCJOgJixKnqawdiQ5aCsaH1ACxyDI
RREmjJv1AgMBAAECggEBAIXP5RrvVgQGry0cSe/1k/RvECQV9o7fTpV5rKpAyXRz
2lhmehBj0hCtsO4AUaM6V4gwzmeRzsByUQroi2wd9I6S3VCkywGHtTzrTlkrU/oy
PLK6RkDWbVPe2b/vSFpaGPtXqBDsvxNIrbkvbNvrRxA3gZETfJKovUf0bswQPtA+
CJb7fVOQUQUH7YE4Mnn0cp/xYnCsX7fO3LISK8sr2Bu9YeVwr4w8JGkYzZq92Gnm
3+5D65vJwy156G2J0H9/jQ2xmWfHb08A0Phw5tH73Bxhq4nqOiA0WPW2MP41V2bU
DAo5vZfxWOaA+vtd+Fyj8TXDD+01rnT0lQicQBloYsECgYEA9ab1xOEb09oUGNSP
vEqTXo4UEZ0GIQSV+gimOKgwl6WjBGoesT9Pl6lMWFBNSqgcuIh39TfrKdWU6L4g
AnyCNMezmT2mIk9OsvajncTDXmayCrGEeVUBJjpvIBOFEfq1LKhvtefFprYrh1iv
LQcuWB3vm5O8wPTSGnYYejY/lekCgYEA4EsUR8CcQtJodKrSuhUVQdAeRabvw59k
Cpj2vrhbtAAnGd3h88V3GTAomMAjozaph/W/t9Hj/kSgOwEY7RShzC/Ispn10MGd
fM8zzNpjRQ3QV14aAmCOgSjYBnNVZy9syWCI+vhUKQQ0T7KPlqV4naQyMei9IlUf
WIsJynXB8i0CgYEA8lif2Ha4rF1WPFTf5MOx9bO3atT96e8LQtvrmLOdFM6tL7q9
wuGR1S6JigrW5dNKWmcP6VUcZG44dRC0pZ9OGWH29jRSxUB2hKo8Knamw2gPN+t/
pO/OrXcykHORcXZj7MXTR6N7ZAgOk31TfNm7x+TosRSkP7yH1uExLsh8HJkCgYEA
o2tQnW7i63TukoKcAuS1ljwSsmENNJT/iREmRpOAjUfHNXuK09g+DGYgThtAW/zJ
zLvrMf2Fho9tx0/nX2W29VSjgC3ZAn7vRi6Z6Vn9FiUYdW+kt7KuYcDmlNyXjxTo
yFjDtUgV2cSscJ0DLZnmpcUqpV6T313x9stnGNJvuwkCgYBwfbewTjPFuLx5L7bX
aIsxYUzvnW7jKTSxZOiGW7zkeEpA/pQpCcF4IeMa2MH+rOLy5fjULO0djl8LHVVf
fwLs/C+jl5C+K27uz9gVCwLshpCOHUGLtqSV1pAatG/ni4DJP56+NjWrbR6VmzGR
M9Y9/8uMkme8zGNr69REvW1C5Q==
-----END PRIVATE KEY-----`;


const publicKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1zoibCr7AebdGpkeeSer
UfWmRqHWSYvAcqaO/yRgL25U2Ej/LRW3X6koBQi7GYVNGy5/JEDUBZX6+z9iGqxH
gHrqn2vMa4dcrPUAHbc6Gb+U3w8Y9l3Nugkbku/HkLvnDbz/CT2o3rZInbTFDOfh
T77FNX+jwahRBxXRleot0UStelbgrwYcw5aRJhqSiMRJfdMO1mFus1KsAvotbjEq
NTK2ZMTZ00cw85Nq8ruDeaqujNfMmfVZh+Zj3bIU24RcU6vWyg5d1tUXPXDGjrtd
utZ/PGBfGZYx//dc8ETNJ4pphxQ1ZAiToCYsSp6msHYkOWgrGh9QAscgyEURJoyb
9QIDAQAB
-----END PUBLIC KEY-----`;

const generateJwt = (address) => {
  const token = jwt.sign(
    {
      iss: "https://static.fansland.io/json/jwk.json",
      //   exp: Math.floor((Date.now() + 1000 * 3600 * 24 * 365) / 1000),
      exp: Math.floor((Date.now() + 1000 * 30 * 60) / 1000),
      iat: Math.floor(Date.now() / 1000),
      aud: "www.example.com",
      sender: address,
      sub: "a80b1ece-e68d-11ee-95e0-6bcefe10fe58",
      jwt_id: uuid.v1(),
      //   jwt_id: "7b0e2b04-faa3-48f0-b477-38a641a87467:166822658140065792"
    },
    privateKey,
    { algorithm: "RS256" }
  );

  return token;
};

// export default generateJwt;

token = generateJwt("hellloworld");
console.log(token);

// //==========
// let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJ3d3cuZXhhbXBsZS5jb20iLCJleHAiOjE3MTEwMTc4NDAsImlhdCI6MTcxMTAxNzU0MCwiaXNzIjoiaHR0cHM6Ly9zdGF0aWMuZmFuc2xhbmQuaW8vanNvbi9qd2suanNvbiIsImp3dF9pZCI6InlvdXJfand0X2lkIiwic2VuZGVyIjoieW91cl9hZGRyZXNzIiwic3ViIjoiYTgwYjFlY2UtZTY4ZC0xMWVlLTk1ZTAtNmJjZWZlMTBmZTU4In0.UagO6-KuCrr51pbw_3uGL5Z7AzaN7DN379D85OG7UMo9LmLFbEGw9xJzHnEkZHBijdUWrowy__6C4CyKDL26jEPpY6ilumdJUn-30eyiU2RhkYG0MfGX3E17488w3Jl_EJoCgCoULaSELXFxthI0nJ7xooZlYht6L9vyDvS-2d6JQ_AfD-wFfFPLUyxFxhzoyQH-3wQ-L6ku4orMf7wh12gu_hgxndBe7AgFAKpe-_Ubxi0HPyYZ8XTBIGrPoNdoBvvkEd0hH4wZFF96HTRkPQFSQX_iELEWa8BhvJGuUctscNnQEuQ7dqJBkRsEx8qM4P73ryxHRRU5j1Ul7Qv8TQ";
// const d = jwt.verify(token, publicKey);
// console.log(d);