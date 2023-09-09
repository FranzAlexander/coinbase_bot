use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use dotenv::dotenv;
use futures::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use model::OneMinuteCandle;
use serde_json::{json, Value};
use sha2::Sha256;

use tokio::{net::TcpStream, signal, sync::mpsc, time::Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::model::Trade;

type HmacSha256 = Hmac<Sha256>;

mod model;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // WebSocket URL for Coinbase Pro
    let url = url::Url::parse("wss://advanced-trade-ws.coinbase.com").unwrap();

    // Establish WebSocket connection
    let (mut ws_stream, _) = connect_async(url).await.unwrap();

    // Channels to subscribe to
    let channels = vec!["heartbeats", "market_trades", "level2"];
    for channel in channels.iter() {
        subscribe(&mut ws_stream, "XRP-USD", channel).await;
    }

    // Shared state to check for shutdown
    let is_terminating = Arc::new(AtomicBool::new(false));

    // Start listening for shutdown signals in the background
    let termination_flag = is_terminating.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        termination_flag.store(true, Ordering::Relaxed);
    });

    // Create channels for market trade and level2 data
    let (market_trades_sender, market_trades_receiver) = mpsc::channel::<Vec<Trade>>(100);
    let (l2_data_sender, l2_data_receiver) = mpsc::channel::<Value>(100);

    // Spawn tasks to process data from the channels
    tokio::spawn(async move {
        process_market_trades_stream(market_trades_receiver).await;
    });

    tokio::spawn(async move {
        process_l2_data_stream(l2_data_receiver).await;
    });

    while let Some(msg) = ws_stream.next().await {
        if is_terminating.load(Ordering::Relaxed) {
            println!("Received shutdown signal. Gracefully terminating...");
            break;
        }

        if let Ok(Message::Text(text)) = msg {
            let message: model::Message = serde_json::from_str(&text).unwrap();

            match message.channel.as_str() {
                "market_trades" => {
                    let _ = market_trades_sender.send(message.events).await;
                }
                "l2_data" => {}
                _ => {}
            }
        }

        let socket_msg = msg.unwrap();
        match socket_msg {
            Message::Text(text) => {
                // Deserialize the received JSON message
                let data: Value = serde_json::from_str(&text).unwrap();

                if let Some(channel_name) = data.get("channel").and_then(Value::as_str) {
                    match channel_name {
                        "market_trades" => {
                            // Send market trade data to the processing task
                            let _ = market_trades_sender.send(data).await;
                        }
                        "l2_data" => {
                            // Send level2 data to the processing task
                            let _ = l2_data_sender.send(data).await;
                        }
                        _ => {}
                    }
                }
            }
            Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {}
        }
    }
}

async fn process_market_trades_stream(mut receiver: mpsc::Receiver<Vec<Trade>>) {
    let mut trades_buffer: VecDeque<Trade> = VecDeque::new();
    let mut start_time = Instant::now();

    while let Some(data) = receiver.recv().await {
        let trades: Vec<Trade> =
            serde_json::from_value(data["events"][0]["trades"].clone()).unwrap();

        for trade in trades {
            trades_buffer.push_back(trade);
        }

        if start_time.elapsed().as_secs() >= 60 {
            if !trades_buffer.is_empty() {
                let candle = OneMinuteCandle::from_trades(trades_buffer.make_contiguous());
                println!(
                    "Candle: Open: {}, Close: {}, High: {}, Low: {}, Volume: {}",
                    candle.open, candle.close, candle.high, candle.low, candle.volume
                );

                trades_buffer.clear();
                start_time = Instant::now();
            }
        }
    }
}

async fn process_l2_data_stream(mut receiver: mpsc::Receiver<Value>) {
    while let Some(data) = receiver.recv().await {
        // println!("{}", data);
    }
}

async fn subscribe(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    market: &str,
    channel: &str,
) {
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

    ws_stream
        .send(Message::Text(subscribe_msg.to_string()))
        .await
        .unwrap();
}

fn sign_message(message: &str) -> String {
    let api_secret = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC can take key of any size");

    mac.update(message.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}
