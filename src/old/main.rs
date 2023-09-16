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
use model::{CandlestickEvent, Level2Event, MarketTradeEvent};
use serde_json::{json, Value};

use tokio::{net::TcpStream, signal, sync::mpsc, task::spawn_blocking, time::Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{event, info, instrument, Level};
use trading_bot::{TradeSignal, TradingBot};
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

    let account = Arc::new(Mutex::new(Account::new()));
    {
        let mut locked_acc = account.lock().unwrap();
        locked_acc.get_wallet().await;
        locked_acc.update_balances().await;
    }
    // WebSocket URL for Coinbase Pro
    let url = url::Url::parse("wss://advanced-trade-ws.coinbase.com").unwrap();

    // Establish WebSocket connection
    let (mut ws_stream, _) = connect_async(url).await.unwrap();

    // Channels to subscribe to
    let channels = vec!["heartbeats", "candles", "level2"];
    for channel in channels.iter() {
        subscribe(&mut ws_stream, "XRP-USD", channel, "subscribe").await;
    }

    let trading_bot = Arc::new(Mutex::new(TradingBot::new()));

    // // Shared state to check for shutdown
    let is_terminating = Arc::new(AtomicBool::new(false));

    // Start listening for shutdown signals in the background
    let termination_flag = is_terminating.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        termination_flag.store(true, Ordering::Relaxed);
    });

    let (candle_sender, candle_receiver) = mpsc::channel::<Vec<CandlestickEvent>>(100);
    let (l2_data_sender, l2_data_receiver) = mpsc::channel::<Vec<Level2Event>>(100);

    let candlestick_trading_bot = trading_bot.clone();
    let l2_trading_bot = trading_bot.clone();

    // // Spawn tasks to process data from the channels
    tokio::spawn(async move {
        process_candle_stick_stream(candle_receiver, candlestick_trading_bot).await;
    });

    tokio::spawn(async move {
        process_l2_data_stream(l2_data_receiver, l2_trading_bot).await;
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
            // info!("MSG: {}", text);
            let event: Event = serde_json::from_str(&text).unwrap();

            match event {
                Event::Subscriptions(_) => {
                    event!(Level::INFO, "Subscription");
                }
                Event::Heartbeats(_) => {
                    // event!(Level::INFO, "Heartbeat");
                }
                Event::Candles(candles) => {
                    let _ = candle_sender.send(candles.events).await;
                }
                Event::L2Data(l2_data) => {
                    let _ = l2_data_sender.send(l2_data.events).await;
                }
            }
        }

        {
            let mut locked_bot = trading_bot.lock().unwrap();
            let locked_acc = account.lock().unwrap();

            if locked_bot.can_trade {
                let signal = locked_bot.check_trade_signal();
                match signal {
                    trading_bot::TradeSignal::Buy => {
                        info!("BUYING");
                        if locked_bot.can_trade && locked_acc.can_buy() {
                            info!("Buy: {}", locked_bot.current_price);
                        }
                    }
                    trading_bot::TradeSignal::Sell => {
                        info!("SELLING");

                        if locked_bot.can_trade && locked_acc.can_sell() {
                            info!("Sell: {}", locked_bot.current_price);
                        }
                    }
                    trading_bot::TradeSignal::Hold => {}
                }
                locked_bot.can_trade = false;
            }
        }
    }

    println!("ALL DONE");
}

async fn process_candle_stick_stream(
    mut receiver: mpsc::Receiver<Vec<CandlestickEvent>>,
    trading_bot: Arc<Mutex<TradingBot>>,
) {
    while let Some(mut candle_event) = receiver.recv().await {
        if let Some(candlestick) = candle_event.first_mut() {
            if candlestick.event_type == "snapshot" {
                if let Some(last_candle) = candlestick.candles.last() {
                    let last_candle_start = last_candle.start;
                    candlestick.candles.retain(|x| x.start == last_candle_start);

                    let mut locked_bot = trading_bot.lock().unwrap();
                    for stick in candlestick.candles.iter() {
                        locked_bot.process_data(IndicatorType::Candlestick(stick.clone()));
                    }
                }
            } else {
                let mut locked_bot = trading_bot.lock().unwrap();
                for stick in candlestick.candles.iter() {
                    locked_bot.process_data(IndicatorType::Candlestick(stick.clone()));
                }
            }
        }
    }
}

async fn process_l2_data_stream(
    mut receiver: mpsc::Receiver<Vec<Level2Event>>,
    trading_bot: Arc<Mutex<TradingBot>>,
) {
    while let Some(l2_event) = receiver.recv().await {
        let mut locked_bot = trading_bot.lock().unwrap();
        locked_bot.process_data(IndicatorType::L2Data(l2_event[0].clone()));
    }
}
