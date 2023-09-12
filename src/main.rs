use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use dotenv::dotenv;
use futures::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use indicators::BotIndicator;
use model::{MarketTradeEvent, OneMinuteCandle};
use serde_json::{json, Value};
use sha2::Sha256;

use tokio::{net::TcpStream, signal, sync::mpsc, time::Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{event, info, instrument, Level};

use crate::model::{Event, Trade};

type HmacSha256 = Hmac<Sha256>;

mod account;
mod indicators;
mod model;

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv().ok();

    // WebSocket URL for Coinbase Pro
    let url = url::Url::parse("wss://advanced-trade-ws.coinbase.com").unwrap();

    // Establish WebSocket connection
    let (mut ws_stream, _) = connect_async(url).await.unwrap();

    // Channels to subscribe to
    let channels = vec!["heartbeats", "market_trades"];
    for channel in channels.iter() {
        subscribe(&mut ws_stream, "XRP-USD", channel, "subscribe").await;
    }

    let bot_indicator = Arc::new(Mutex::new(BotIndicator::new()));
    let bot_clone = bot_indicator.clone();
    // Shared state to check for shutdown
    let is_terminating = Arc::new(AtomicBool::new(false));

    // Start listening for shutdown signals in the background
    let termination_flag = is_terminating.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        termination_flag.store(true, Ordering::Relaxed);
    });

    // Create channels for market trade and level2 data
    let (market_trades_sender, market_trades_receiver) = mpsc::channel::<MarketTradeEvent>(100);
    let (l2_data_sender, l2_data_receiver) = mpsc::channel::<Value>(100);

    // Spawn tasks to process data from the channels
    tokio::spawn(async move {
        process_market_trades_stream(market_trades_receiver, bot_clone).await;
    });

    tokio::spawn(async move {
        process_l2_data_stream(l2_data_receiver).await;
    });

    while let Some(msg) = ws_stream.next().await {
        if is_terminating.load(Ordering::Relaxed) {
            info!("Received shutdown signal. Gracefully terminating...");
            for channel in channels.iter() {
                subscribe(&mut ws_stream, "XRP-USD", channel, "unsubscribe").await;
            }

            break;
        }

        if let Ok(Message::Text(text)) = msg {
            let event: Event = serde_json::from_str(&text).unwrap();

            match event {
                Event::Subscriptions(_) => {
                    event!(Level::INFO, "Subscription");
                }
                Event::Heartbeats(_) => {
                    event!(Level::INFO, "Heartbeat");
                }
                Event::MarketTrades(trades) => {
                    let _ = market_trades_sender.send(trades.events[0].clone()).await;
                }
            }
        }
        {
            let bot_locked = bot_indicator.lock().unwrap();
            if let Some(short_ema) = bot_locked.get_short_ema() {
                info!("Short ema: {}", short_ema);
            }
            if let Some(long_ema) = bot_locked.get_long_ema() {
                info!("Long ema: {}", long_ema);
            }

            if let Some(hista) = bot_locked.get_macd_histogram() {
                info!("Histagram: {}", hista);
            }
            if let Some(rsi) = bot_locked.rsi.get_rsi() {
                info!("RSI: {}", rsi);
            }
            if let Some(adx) = bot_locked.adx.get_adx() {
                info!("ADX: {}", adx);
            }
        }
    }
}

async fn process_market_trades_stream(
    mut receiver: mpsc::Receiver<MarketTradeEvent>,
    bot: Arc<Mutex<BotIndicator>>,
) {
    let mut trade_buffer: Vec<Trade> = Vec::new();
    let mut start_time = Instant::now();

    while let Some(data) = receiver.recv().await {
        if data.event_type == "update" {
            for trade in data.trades {
                trade_buffer.push(trade);
            }
        }

        if start_time.elapsed().as_secs() >= 60 && !trade_buffer.is_empty() {
            let candle = OneMinuteCandle::from_trades(trade_buffer.as_slice());
            info!(
                "Candle: Open: {}, Close: {}, High: {}, Low: {}, Volume: {}",
                candle.open, candle.close, candle.high, candle.low, candle.volume
            );

            let bot_clone = bot.clone();
            tokio::task::spawn_blocking(move || {
                let mut bot_locked = bot_clone.lock().unwrap();
                bot_locked.process_data(indicators::IndicatorType::Candlestick(candle));
            })
            .await
            .expect("Failed to process in blocking thread");

            trade_buffer.clear();
            start_time = Instant::now();
        }
    }
}

async fn process_l2_data_stream(mut receiver: mpsc::Receiver<Value>) {
    while let Some(_data) = receiver.recv().await {}
}
async fn subscribe(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    market: &str,
    channel: &str,
    event: &str,
) {
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

fn sign_message(message: &str) -> String {
    let api_secret = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC can take key of any size");

    mac.update(message.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}
