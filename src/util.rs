use futures::SinkExt;
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;
use sha2::Sha256;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::coin::CoinSymbol;

type HmacSha256 = Hmac<Sha256>;

pub async fn subscribe(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    market: &str,
    event: &str,
) {
    let channels = vec!["heartbeats", "market_trades"];
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

pub fn market_subcribe_string(coin_symbol: &str, curreny_sybmol: &str) -> String {
    format!("{}-{}", coin_symbol, curreny_sybmol)
}

fn sign_message(message: &str) -> String {
    let api_secret = std::env::var("API_SECRET").expect("API_SECRET not found in environment");

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

pub fn create_headers(
    secret_key: &[u8],
    api_key: &str,
    method: &str,
    request_path: &str,
    body: &str,
) -> HeaderMap {
    let timestamp = format!("{}", chrono::Utc::now().timestamp());

    let signature = http_sign(secret_key, &timestamp, method, request_path, body);

    let mut headers = HeaderMap::new();

    headers.insert("CB-ACCESS-KEY", HeaderValue::from_str(api_key).unwrap());
    headers.insert("CB-ACCESS-SIGN", HeaderValue::from_str(&signature).unwrap());
    headers.insert(
        "CB-ACCESS-TIMESTAMP",
        HeaderValue::from_str(&timestamp).unwrap(),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    headers
}

#[inline]
pub fn get_api_string(symbol: CoinSymbol, currency: CoinSymbol, endpoint: &str) -> String {
    format!(
        "{}/{}-{}",
        endpoint,
        String::from(symbol),
        String::from(currency)
    )
}
