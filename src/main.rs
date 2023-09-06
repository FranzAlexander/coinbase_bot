use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::model::OneMinuteCandle;
use dotenv::dotenv;
use futures::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use indicators::BotIndicator;
use model::Trade;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{json, Value};
use sha2::Sha256;
use tokio::sync::mpsc;
use tokio::{net::TcpStream, signal};
use tokio_tungstenite::{
    connect_async,
    stream::Stream,
    tungstenite::{client::AutoStream, Message},
    MaybeTlsStream, WebSocketStream,
};

mod indicators;
mod model;

type HmacSha256 = Hmac<Sha256>;

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

    // Initialize candlestick data
    let mut current_canlde = OneMinuteCandle::default();

    let mut bot_indcator = BotIndicator::new();

    // Shared state to check for shutdown
    let is_terminating = Arc::new(AtomicBool::new(false));

    // Start listening for shutdown signals in the background
    let termination_flag = is_terminating.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        termination_flag.store(true, Ordering::Relaxed);
    });

    // Create channels for market trade and level2 data
    let (market_trades_sender, market_trades_receiver) = mpsc::channel::<Value>(100);
    let (l2_data_sender, l2_data_receiver) = mpsc::channel::<Value>(100);

    // Spawn tasks to process data from the channels
    tokio::spawn(async move {
        process_market_trades_stream(market_trades_receiver, &mut bot_indcator).await;
    });

    tokio::spawn(async move {
        process_l2_data_stream(l2_data_receiver).await;
    });

    // Main loop to process WebSocket messages
    while let Some(msg) = ws_stream.next().await {
        if is_terminating.load(Ordering::Relaxed) {
            println!("Received shutdown signal. Gracefully terminating...");
            break;
        }
        let socket_msg = msg.expect("Error reading message");
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

// Process market trade data from the channel
async fn process_market_trades_stream(
    mut receiver: mpsc::Receiver<Value>,
    bot_indicator: &mut BotIndicator,
) {
    let mut current_candle = OneMinuteCandle::default();

    while let Some(data) = receiver.recv().await {
        let trades: Vec<Trade> =
            serde_json::from_value(data["events"][0]["trades"].clone()).unwrap();

        println!("{:?}", trades);

        for trade in trades {
            update_candle_with_trade(&trade, &mut current_candle, bot_indicator)
        }
    }
}

fn update_candle_with_trade(
    trade: &Trade,
    current_candle: &mut OneMinuteCandle,
    bot_indicator: &mut BotIndicator,
) {
    let price = trade.price;
    let size = trade.size;
    let time = trade.time;

    if current_candle.open.is_none() {
        current_candle.open = Some(price);
        current_candle.start_time = Some(time);
        current_candle.end_time = Some(time + chrono::Duration::minutes(1));
        current_candle.high = Some(price);
        current_candle.low = Some(price);
    }

    if price > current_candle.high.unwrap() {
        current_candle.high = Some(price);
    }

    if price < current_candle.low.unwrap() {
        current_candle.low = Some(price);
    }

    current_candle.close = Some(price);
    current_candle.volume += size;

    if time >= current_candle.end_time.unwrap() {
        println!("Completed Candle: {:?}", current_candle);
        let complete_candle = current_candle.clone();
        bot_indicator.send_to_processing(indicators::IndicatorType::Candlestick(complete_candle));
        bot_indicator.check_signal();

        *current_candle = OneMinuteCandle::default();
    }
}

async fn process_l2_data_stream(mut receiver: mpsc::Receiver<Value>) {
    while let Some(data) = receiver.recv().await {
        // println!("{}", data);
    }
}
