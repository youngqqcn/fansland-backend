const jwt = require("jsonwebtoken");
const uuid  = require('uuid');

const privateKey = `-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCq+2Q1kb+yO/sq
ALMG6tZglPnZhBZyxbkoR/Gk5fPsgiYQN9w6B8mc9QKIBapriSkqZUG0bnv3xb3o
tZtAMgapVKFTLv3ZndZr/1RUYF3uDd3BOgOkYRS2PDpBKVZEyW9O+goIQIekZph3
EQ7bAneXSw5/npHm93iHLyN+w9fH5id2Mwl/6REfotQo6zBE0erqE2lPhVZqeUA1
omyRIp0K502m7eeztaiYhACo8o4BYQsGbJmzOb6cWYomTgZMewByl67K39yiQ+U3
IY91G/xcEG4bsRr9t6plvWFujeWEcsA6tuxw8+/xkYyKBrIZwz55nrby4k8lGrlm
+194OkZtAgMBAAECggEAXVsLePBOcXushF+DYGiCipNs4+Xp99qbYTH+72Ea0Oyy
6bIGTa0I2Okx66DkTvjCAELvIBwfFcbjDRdzYBawam4p82g59ELo22i626MORjdN
9/28FVloXKP/zqpKTx6I5t9A8QaCyTG3V3N53/y13WZ+0RF8alZ+eZk0UTePLrZG
+IrDGomu0g08TXkiMzS55mxv0UAFXLTtETUHDI+jFDOuhpLtu25PRTbYdElExu7w
hHbS1KQ2g+s/LuZ+4IMkBakhR8b7yKoawUCGhXFXMwUHt/v4gXTMeWn/9VpYQO88
3VPJ+mOrduiAyVZxpk3vBfEBUZH5Zt3QIh95CXe5xQKBgQDnpnTBRbg5FcVNAdyk
H0+Nm2UFFBCmRF00VsonkbvRC0fY5n5b/pWoI29jhIXKC5QFgdCZIAUAHFTntmRI
AhfNwQSvXVy8ztFFhq1ZeVncslLwsTijfkzvVo7LB1XlLE7xGGal5qXou66MHch3
7YlElAa70S1Ng1lleIrez35zrwKBgQC89GV2/+KjbZ3PThvSJfXiFsEP9voEZiqg
2W2plP3gHDhWPK/ugUN+/2n7BOtqnAY+vBS01kMqfDMOwFCNIGnY/sntq/AsHMA3
MvlLqwuSVfDlG/n9n00ZLcx4vxISiRfBc3sZPtoCTG4RX1IlDFB8+ka/+8XwzmVl
XA1k0LTCowKBgBkAJP+q55v5loaeGdL4shxFVhy7MqTPIgQBDeZBWzTPc9yb261Q
B5TS5jWmWs4Ye8wwW3P7Oa7uX9d2HtKKr0j8eOX4PIcAByPqyhCrASDJehwR9Fvo
yKLYA6czznhiVM6+ZG2pO+SnRMpIeJdA4pytISDqDWqdL2u9G7e6SxpvAoGBAJYm
Z1KOuBxCCPspCHaP77n/dt8m1ToVrZ4v6TKu0Kb8BdCRXQRb97S9Zgwwtpn67gOA
Fbw13x2toC0CqX/b2AC1RuT0kl+bYSr7+Jomi4V3gXuuJZNiuTNe73Kr8sKD6jqx
d4pyJW9aKMTpSC9kf7kwcHQyr/JRMMXAxmvsZRedAoGAUfpRwgFb8yL8bmq3q7UU
EMVscaHcEZxHwB5sL4vvCXQEU3WLom302gGw00psXsJZkTvHmXJytyWaAbc2EvaQ
Y7Z0aoz0DgA968vxj/pNmmi/HcwpsbMG61REBWNj5vFZBEKkOvPwRoFVhJid7sYr
IP+4mS/xTh8NpNqSBMbwJWs=
-----END PRIVATE KEY-----`;


const publicKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAqvtkNZG/sjv7KgCzBurW
YJT52YQWcsW5KEfxpOXz7IImEDfcOgfJnPUCiAWqa4kpKmVBtG5798W96LWbQDIG
qVShUy792Z3Wa/9UVGBd7g3dwToDpGEUtjw6QSlWRMlvTvoKCECHpGaYdxEO2wJ3
l0sOf56R5vd4hy8jfsPXx+YndjMJf+kRH6LUKOswRNHq6hNpT4VWanlANaJskSKd
CudNpu3ns7WomIQAqPKOAWELBmyZszm+nFmKJk4GTHsAcpeuyt/cokPlNyGPdRv8
XBBuG7Ea/beqZb1hbo3lhHLAOrbscPPv8ZGMigayGcM+eZ628uJPJRq5ZvtfeDpG
bQIDAQAB
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