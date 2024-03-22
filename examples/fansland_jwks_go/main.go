package main

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/dgrijalva/jwt-go"
	"github.com/google/uuid"
)

const (
	JWT_PRIVATE_KEY string = `-----BEGIN PRIVATE KEY-----
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
-----END PRIVATE KEY-----`
)

func main() {
	address := "0xaaaabbbbcccceddd" // 自定义的

	// 创建 JWT Token
	token := jwt.New(jwt.SigningMethodRS256)

	privateKey, err := jwt.ParseRSAPrivateKeyFromPEM([]byte(JWT_PRIVATE_KEY))
	if err != nil {
		fmt.Printf("parse error: %v", err)
		return
	}

	// 设置 JWT Token 中的声明
	claims := token.Claims.(jwt.MapClaims)
	claims["iss"] = "https://static.fansland.io/json/jwk.json"
	claims["exp"] = time.Now().Add(time.Minute * 5).Unix() // TODO: 过期时间
	claims["iat"] = time.Now().Unix()
	claims["aud"] = "fansland.io"
	claims["sub"] = "a80b1ece-e68d-11ee-95e0-6bcefe10fe58"
	claims["jwt_id"] = uuid.New() // TODO: 这里用用户ID代替
	claims["address"] = address

	// 使用私钥对 JWT Token 进行签名
	tokenString, err := token.SignedString(privateKey)
	if err != nil {
		fmt.Printf("sign error: %v", err)
		return
	}

	x, _ := json.Marshal(claims)
	fmt.Printf("%v\n", string(x))

	fmt.Println("JWT Token:", tokenString)
}
