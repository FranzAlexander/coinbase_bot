use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use futures::StreamExt;
use model::{candlestick::Candlestick, event::MarketTradeEvent};

use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::spawn_blocking,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{event, info, Level};
use url::Url;

use crate::{account::BotAccount, model::event::Event, trading_bot::TradingBot, util::subscribe};

mod account;
mod indicators;
mod model;
mod trading_bot;
mod util;

const RECONNECTION_DELAY: u64 = 3;

#[tokio::main]
async fn main() -> Result<()> {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber)?;

    let mut bot_account = BotAccount::new();
    bot_account.update_balances().await;

    let trading_bot = Arc::new(Mutex::new(TradingBot::new()));

    // Create a channel for sending and receiving MarketTradeEvent objects with a buffer size of 250.
    let (tx, mut rx) = mpsc::channel::<Vec<MarketTradeEvent>>(100);
    let (tradeing_bot_tx, mut trading_bot_rx) = mpsc::channel::<Candlestick>(10);

    // Parse the WebSocket URL for the Coinbase exchange.
    let url = Url::parse("wss://advanced-trade-ws.coinbase.com")
        .context("Failed to create coinbase url")?;

    // Create a shared AtomicBool flag to control when to keep running various components.
    let keep_running = Arc::new(AtomicBool::new(true));

    // Clone the keep_running flag for use in the WebSocket component.
    let candle_going = keep_running.clone();
    let trading_bot_keep_going = keep_running.clone();

    // Spawn a blocking thread to run the candle function with the provided parameters.
    let blocking_handler = spawn_blocking(move || candle(&mut rx, tradeing_bot_tx, candle_going));
    let trading_bot_handler = spawn_blocking(move || {
        trading_bot_run(&mut trading_bot_rx, trading_bot, trading_bot_keep_going)
    });

    // Clone the keep_running flag for use in the WebSocket component.
    let websocket_keep_running = keep_running.clone();

    // Spawn a Tokio async task to run the WebSocket component with the provided parameters.
    let join_handler = tokio::spawn(async move { run(url, tx, websocket_keep_running).await });

    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal. Gracefully terminating...");

    keep_running.store(false, Ordering::SeqCst);
    join_handler.abort();
    blocking_handler.abort();
    trading_bot_handler.abort();

    Ok(())
}

async fn run(
    ws_url: Url,
    tx: Sender<Vec<MarketTradeEvent>>,
    keep_running: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    while keep_running.load(Ordering::Relaxed) {
        match connect_async(ws_url.clone()).await {
            Ok((mut ws_stream, _)) => {
                info!("Connected to server!");

                subscribe(&mut ws_stream, "BTC-USD", "subscribe").await;
                while let Some(msg) = ws_stream.next().await {
                    if let Ok(Message::Text(text)) = msg {
                        let event: Event = serde_json::from_str(&text)
                            .context("failed to parse message")
                            .unwrap();

                        match event {
                            Event::Subscriptions(_) => {}
                            Event::Heartbeats(_) => {}
                            Event::MarketTrades(market_trades) => {
                                let _ = tx.send(market_trades).await;
                            }
                            Event::Ticker(ticker) => {
                                info!("TICKER: {:?}", ticker);
                            }
                        }
                    }
                }
                event!(
                    Level::WARN,
                    "Connection closed, reconnecting in 3 seconds..."
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(RECONNECTION_DELAY)).await;
            }
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Failed to connect to {}: {}. Retrying in  seconds...",
                    ws_url,
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(RECONNECTION_DELAY)).await;
            }
        }
    }

    Ok(())
}

fn candle(
    rx: &mut Receiver<Vec<MarketTradeEvent>>,
    tx: Sender<Candlestick>,
    keep_running: Arc<AtomicBool>,
) {
    let mut candlestick: Candlestick = Candlestick::new(Utc::now());

    while keep_running.load(Ordering::Relaxed) {
        while let Some(market_trades) = rx.blocking_recv() {
            for trade_event in market_trades.iter() {
                if trade_event.event_type == "snapshot" {
                    let end_time = trade_event.trades[0].time;
                    let start_time = get_start_time(&end_time);

                    candlestick = Candlestick::new(start_time);

                    for trade in trade_event.trades.iter() {
                        if trade.time >= start_time && trade.time <= end_time {
                            candlestick.update(trade.price, trade.size);
                        }
                    }
                } else {
                    for trade in trade_event.trades.iter() {
                        if trade.time > candlestick.end {
                            let _ = tx.blocking_send(candlestick);
                            let start_time = get_start_time(&trade.time);
                            candlestick = Candlestick::new(start_time);
                        }

                        candlestick.update(trade.price, trade.size);
                    }
                }
            }
        }
    }
}

fn get_start_time(end_time: &DateTime<Utc>) -> DateTime<Utc> {
    let start_time: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(end_time.year(), end_time.month(), end_time.day()).unwrap(),
            NaiveTime::from_hms_opt(end_time.hour(), end_time.minute(), 0).unwrap(),
        ),
        Utc,
    );

    start_time
}

fn trading_bot_run(
    rx: &mut Receiver<Candlestick>,
    trading_bot: Arc<Mutex<TradingBot>>,
    keep_running: Arc<AtomicBool>,
) {
    while keep_running.load(Ordering::Relaxed) {
        if let Some(candlestick) = rx.blocking_recv() {
            println!("{:?}", candlestick);
            let mut locked_bot = trading_bot.blocking_lock();
            locked_bot.update_bot(candlestick);
        }
    }
}
