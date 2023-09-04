use dotenv::dotenv;
use futures::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use tokio_tungstenite::{connect_async, tungstenite::Message};

mod model;

type HmacSha256 = Hmac<Sha256>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let url = url::Url::parse("wss://advanced-trade-ws.coinbase.com").unwrap();
    let heartbeat_channel = subscribe("XRP-USD", "heartbeats");
    println!("{heartbeat_channel}");

    let (mut ws_stream, _) = connect_async(url).await.unwrap();

    ws_stream
        .send(Message::Text(heartbeat_channel))
        .await
        .unwrap();

    let ticker_channel = subscribe("XRP-USD", "candles");

    ws_stream.send(Message::Text(ticker_channel)).await.unwrap();

    while let Some(msg) = ws_stream.next().await {
        let socket_msg = msg.expect("Error reading message");
        println!("{}", socket_msg)
    }
}

fn subscribe(market: &str, channel: &str) -> String {
    let timestamp = format!("{}", chrono::Utc::now().timestamp());
    let msg_to_sign = format!("{}{}{}", timestamp, channel, market);
    let signature = sign_message(&msg_to_sign);
    let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");

    let subscribe_msg = json!({
        "type": "subscribe",
        "product_ids": [market],
        "channel": channel,
        "api_key": api_key,
        "timestamp": timestamp,
        "signature": signature
    });

    subscribe_msg.to_string()
}

fn sign_message(message: &str) -> String {
    let api_secret = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC can take key of any size");

    mac.update(message.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}
