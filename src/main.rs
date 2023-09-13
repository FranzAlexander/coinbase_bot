use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use account::Account;
use dotenv::dotenv;
use futures::{SinkExt, StreamExt};
use model::{MarketTradeEvent, OneMinuteCandle};
use serde_json::{json, Value};

use tokio::{net::TcpStream, signal, sync::mpsc, time::Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{event, info, instrument, Level};
use trading_bot::TradingBot;
use util::subscribe;

use crate::{
    model::{Event, Trade},
    trading_bot::IndicatorType,
};

mod account;
mod indicators;
mod model;
mod trading_bot;
mod util;

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let mut account = Account::new();
    account.get_wallet().await;
    account.update_balances().await;

    // WebSocket URL for Coinbase Pro
    let url = url::Url::parse("wss://advanced-trade-ws.coinbase.com").unwrap();

    // Establish WebSocket connection
    let (mut ws_stream, _) = connect_async(url).await.unwrap();

    // Channels to subscribe to
    let channels = vec!["heartbeats", "candles"];
    for channel in channels.iter() {
        subscribe(&mut ws_stream, "XRP-USD", channel, "subscribe").await;
    }

    let trading_bot = Arc::new(Mutex::new(TradingBot::new()));

    // let bot_indicator = Arc::new(Mutex::new(BotIndicator::new()));
    // let bot_clone = bot_indicator.clone();

    // // Shared state to check for shutdown
    let is_terminating = Arc::new(AtomicBool::new(false));

    // Start listening for shutdown signals in the background
    let termination_flag = is_terminating.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        termination_flag.store(true, Ordering::Relaxed);
    });

    // // Create channels for market trade and level2 data
    // let (market_trades_sender, market_trades_receiver) = mpsc::channel::<MarketTradeEvent>(100);
    // let (l2_data_sender, l2_data_receiver) = mpsc::channel::<Value>(100);

    // // Spawn tasks to process data from the channels
    // tokio::spawn(async move {
    //     process_market_trades_stream(market_trades_receiver, bot_clone).await;
    // });

    // tokio::spawn(async move {
    //     process_l2_data_stream(l2_data_receiver).await;
    // });

    while let Some(msg) = ws_stream.next().await {
        if is_terminating.load(Ordering::Relaxed) {
            info!("Received shutdown signal. Gracefully terminating...");
            for channel in channels.iter() {
                subscribe(&mut ws_stream, "XRP-USD", channel, "unsubscribe").await;
            }

            break;
        }

        if let Ok(Message::Text(text)) = msg {
            println!("{}", text);
            let event: Event = serde_json::from_str(&text).unwrap();

            match event {
                Event::Subscriptions(_) => {
                    event!(Level::INFO, "Subscription");
                }
                Event::Heartbeats(_) => {
                    event!(Level::INFO, "Heartbeat");
                }
                Event::Candles(candles) => {
                    if candles.events[0].event_type == "update" {
                        let mut locked_bot = trading_bot.lock().unwrap();
                        locked_bot.process_data(IndicatorType::Candlestick(
                            candles.events[0].candles[0].clone(),
                        ))
                    }
                }
            }
        }

        {
            let mut locked_bot = trading_bot.lock().unwrap();
            if locked_bot.can_trade {
                let signal = locked_bot.check_trade_signal();
                info!("{}", signal);
                locked_bot.can_trade = false;
            }
        }
    }
}

async fn process_market_trades_stream(
    mut receiver: mpsc::Receiver<MarketTradeEvent>,
    bot: Arc<Mutex<TradingBot>>,
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
                // bot_locked.process_data(indicators::IndicatorType::Candlestick(candle));
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
