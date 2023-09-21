use futures::SinkExt;
use hmac::{Hmac, Mac};
use reqwest::header::HeaderMap;
use serde_json::json;
use sha2::Sha256;
use tokio::{net::TcpStream, signal, sync::mpsc, time::Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{event, info, instrument, Level};

type HmacSha256 = Hmac<Sha256>;

pub async fn subscribe(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    market: &str,
    event: &str,
) {
    let channels = vec!["heartbeats", "market_trades", "ticker_batch", "user"];
    for channel in channels.iter() {
        let timestamp = format!("{}", chrono::Utc::now().timestamp());
        let msg_to_sign = format!("{}{}{}", timestamp, channel, market);
        let signature = sign_message(&msg_to_sign);
        let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");

        let subscribe_msg = json!({
            "type": event.to_string(),
            "product_ids": [market],
            "channel": channel,
            "api_key": api_key,
            "timestamp": timestamp,
            "signature": signature
        });

        ws_stream
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .unwrap();
    }
}

fn sign_message(message: &str) -> String {
    let api_secret = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC can take key of any size");

    mac.update(message.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}

pub async fn send_get_request<T: for<'de> serde::Deserialize<'de>>(
    client: &reqwest::Client,
    url: &str,
    headers: HeaderMap,
) -> Result<T, reqwest::Error> {
    client
        .get(url)
        .headers(headers)
        .send()
        .await?
        .json::<T>()
        .await
}

pub fn http_sign(
    secret_key: &[u8],
    timestamp: &str,
    method: &str,
    request_path: &str,
    body: &str,
) -> String {
    let message = format!("{}{}{}{}", timestamp, method, request_path, body);

    let mut mac = HmacSha256::new_from_slice(secret_key).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();

    format!("{:x}", result.into_bytes())
}
