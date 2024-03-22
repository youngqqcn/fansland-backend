package org.example;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HashMap;
import java.util.Map;
import java.util.TimeZone;
import java.util.Date;
import java.text.SimpleDateFormat;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;

import okhttp3.MediaType;
import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.RequestBody;
import okhttp3.Response;

public class Main {

    private static final String API_KEY = "ca17a3e225a85a74290831f504aceec5";

    public static void main(String[] args) {
        Map<String, Object> req = new HashMap<>();
        req.put("chain_id", 1);
        req.put("nft_contract", "0xE9AE3261a475a27Bb1028f140bc2a7c843318afD");
        req.put("nft_owner", "0xF4435c244A292a8E8D56767bf6DF9b9c4D59aEED");
        req.put("nft_token_id", 66);
        req.put("timestamp", getCurrentTimestamp());
        req.put("signature", "");

        // 组成签名消息
        String sigMsg = req.get("chain_id") + "&" +
                req.get("nft_contract") + "&" +
                req.get("nft_owner") + "&" +
                req.get("nft_token_id") + "&" +
                req.get("timestamp") + "&" +
                API_KEY;
        System.out.println("签名消息: " + sigMsg);

        // 签名
        String sig = calculateSHA256(sigMsg.toLowerCase());
        System.out.println("签名结果: " + sig);

        // 设置签名
        req.put("signature", sig);

        // 发起请求
        Gson gson = new GsonBuilder().create();
        String jsonReq = gson.toJson(req);
        MediaType mediaType = MediaType.parse("application/json; charset=utf-8");
        RequestBody requestBody = RequestBody.create(mediaType, jsonReq);
        Request request = new Request.Builder()
                .url("https://fansland.io/openapi-ok/v1/getQRCode")
                .post(requestBody)
                .addHeader("Content-Type", "application/json")
                .build();

        OkHttpClient client = new OkHttpClient();
        try {
            Response response = client.newCall(request).execute();
            String jsonResponse = response.body().string();
            JsonObject jsonObject = gson.fromJson(jsonResponse, JsonObject.class);
            System.out.println("响应结果:");
            System.out.println(jsonObject);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private static String calculateSHA256(String input) {
        try {
            MessageDigest messageDigest = MessageDigest.getInstance("SHA-256");
            byte[] hashBytes = messageDigest.digest(input.getBytes());
            StringBuilder hexString = new StringBuilder();
            for (byte b : hashBytes) {
                String hex = Integer.toHexString(0xff & b);
                if (hex.length() == 1) {
                    hexString.append('0');
                }
                hexString.append(hex);
            }
            return hexString.toString();
        } catch (NoSuchAlgorithmException e) {
            e.printStackTrace();
        }
        return "";
    }

    private static long getCurrentTimestamp() {
        Date date = new Date();
        SimpleDateFormat sdf = new SimpleDateFormat("yyyy-MM-dd HH:mm:ss");
        sdf.setTimeZone(TimeZone.getTimeZone("UTC"));
        String formattedDate = sdf.format(date);
        try {
            Date utcDate = sdf.parse(formattedDate);
            return utcDate.getTime() / 1000L;
        } catch (Exception e) {
            e.printStackTrace();
        }
        return 0L;
    }
}