const jwt = require("jsonwebtoken");
const uuid  = require('uuid');

const privateKey = `-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQClCXDsknZzqW8r
3mwifPWG7SJG9/n9EsXmdGwmeFiub5kSdBCJxCfZwG2HnDJnMWSQtIacHjytRenJ
nlSnNPJFGTT9h3F7AZgl8zUDFkOv0hiNtlLc3m0/MpKwcbRLl9uUUBQXTRooS25F
N/XqzM3SOMxsLQaFwl8aTyEip9GTIq9gvUFfBwqHrSYj9coEc4VJ1er1a913PIAS
i6UFO4or0NdHqnNW4QtYCpTQqArAegUH0inAUSx6e7Qsp1hiE9ACOh/kW8iawFMf
DdFHK8J7o1P7oG5IH/iZeIWVNn2FY3eFLywmiw+71qxltrQxM/JJZ9KS7WWPqbuK
q5/j+o5PAgMBAAECggEBAII+GSgZzfEpXdq5BY2SIBIDQnHPcTIPhq6AYnlZsGK5
H4w4nHScoPZvrgC/e+eES2EHltPN6QS1j/LVSOlMF4YAhpkuMphogbIxZTyybWhr
3YS4xlwYDQ860hj9ItQlW9yH9dMNvf7eyH9H0oMibavhtzIHtoYJBxS0LeozlmkO
SLmIO/GL9w3abyixPR5mp/c1/I3kuBiCoHssiy9u89S1tak88TRKTXLFybMMCTZC
Z+VQiajxB0KaY5obDdUVL8VmuYvu3AhzFbIGspdBBQbElTlITDkJg5W5nH0JcBb3
Z1+tvdcPzZl1zfi9wOsdcJ0QrUgt+lyA6JsRa0ytUIECgYEA4HhELnkwupFZjgXv
4yShtXar7287CwfX6kkO59ZqtRcaj/4LBp8W9VM5zAPgBoq+pQN2W+S7BLoo3uUm
59c1rQJgBoCp5zJG/hCHbjiPm7Z64znlAHadg2/swRl2+9+NuQtjnHfOnoM6/VK6
5bwiaF4z/6vpwGIQfgAmOYPJM2cCgYEAvDgFLva+8lgL79nBIZmDxccOpDEWsohx
4pQBXDpJ3eSh8/qcuE3ZC556jsjzl7NRf+UI2gvsFOJ9A4HgJdw58KhjRXLb2rgw
UrH2i3T9+fxtt4dWbWBPgxCc+0sh15K2MVqG81OX8xiEXRMBOSErOCMHbEzX/u8i
+dlFavqApNkCgYEAteAcdn9RgUl2A4JB/YxXzj1qUGWStHxVcQpjHmv4J+ShbBxN
+L58jqgxg2F8ajCPCgsFIq3w7oKbzQZlAKK6ZgUvovUrNR3iscbYxPDlQyiW53eo
WjjBpZbRgSBN7QUUwuM8gbH9Yyg3arxWnQBT7LavcTBtBKkwbbdXQXDLXG8CgYBT
YCFzr1vgcH5BLl2uK5nCt63Vr+UVCvof215K44srHwda024u+lUmH3q822mEjquQ
kv170YIvsjCcqCCJxOdpor3u2rVnSuYlC65iEl4bKeXUQcBYTSyLq4VOj/aazuOk
MyzKhwItbnPb4vpMu1Ow11uHbvbTeE0vDj8A55yq6QKBgBd6oxk7Fnjyd8rT8WYQ
zdUjSk8XI+GBRRn2RDkNi85KuwExOFY60ZfmJS57V1v7eWV5b1zspnUrBsMrJtN3
iQG5Jy2svBHu0+fZbdXAM9PIplxkzBiUifghHyf5uuQkX0iNvXkY/GaFAPhQ+EU3
P/1wlb/sDtZP+xC/SEDDgb2y
-----END PRIVATE KEY-----`;


const publicKey = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEApQlw7JJ2c6lvK95sInz1
hu0iRvf5/RLF5nRsJnhYrm+ZEnQQicQn2cBth5wyZzFkkLSGnB48rUXpyZ5UpzTy
RRk0/YdxewGYJfM1AxZDr9IYjbZS3N5tPzKSsHG0S5fblFAUF00aKEtuRTf16szN
0jjMbC0GhcJfGk8hIqfRkyKvYL1BXwcKh60mI/XKBHOFSdXq9WvddzyAEoulBTuK
K9DXR6pzVuELWAqU0KgKwHoFB9IpwFEsenu0LKdYYhPQAjof5FvImsBTHw3RRyvC
e6NT+6BuSB/4mXiFlTZ9hWN3hS8sJosPu9asZba0MTPySWfSku1lj6m7iquf4/qO
TwIDAQAB
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