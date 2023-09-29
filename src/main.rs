use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use account::{ADA_SYMBOL, BTC_SYMBOL, ETH_SYMBOL, XRP_SYMBOL};
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use futures::{SinkExt, StreamExt};
use model::{
    account::ActiveTrade,
    candlestick::{candle_snapshot, candle_update, Candlestick, CandlestickMessage},
    event::MarketTradeEvent,
    OrderStatus,
};

use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::spawn_blocking,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, event, info, Level};
use url::Url;

use crate::{
    account::{BotAccount, USD_SYMBOL},
    model::event::Event,
    trading_bot::{TradeSignal, TradingBot},
    util::{market_subcribe_string, subscribe},
};

mod account;
mod coin;
mod indicators;
mod model;
mod trading_bot;
mod util;

const RECONNECTION_DELAY: u64 = 3;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logging()?;

    let trading_bot = Arc::new(Mutex::new(TradingBot::new()));

    // Create a channel for sending and receiving MarketTradeEvent objects with a buffer size of 250.
    let (tx, mut rx) = mpsc::channel::<Vec<MarketTradeEvent>>(150);
    let (tradeing_bot_tx, mut trading_bot_rx) = mpsc::channel::<Candlestick>(10);
    let (bot_signal_tx, mut bot_signal_rx) = mpsc::channel::<TradeSignal>(50);

    // Parse the WebSocket URL for the Coinbase exchange.
    let url = Url::parse("wss://advanced-trade-ws.coinbase.com")
        .context("Failed to create coinbase url")?;

    // Create a shared AtomicBool flag to control when to keep running various components.
    let keep_running = Arc::new(AtomicBool::new(true));

    // Clone the keep_running flag for use in the WebSocket component.
    let candle_going = keep_running.clone();
    let trading_bot_keep_going = keep_running.clone();

    launch_processing_tasks(rx, bot_signal_tx, keep_running.clone());
    launch_websocket_tasks(url, tx, bot_signal_rx, keep_running.clone());

    // // Spawn a blocking thread to run the candle function with the provided parameters.
    // let blocking_handler = spawn_blocking(move || candle(&mut rx, tradeing_bot_tx, candle_going));
    // let trading_bot_handler = spawn_blocking(move || {
    //     trading_bot_run(
    //         &mut trading_bot_rx,
    //         bot_signal_tx,
    //         trading_bot,
    //         trading_bot_keep_going,
    //     )
    // });

    // Clone the keep_running flag for use in the WebSocket component.

    // Spawn a Tokio async task to run the WebSocket component with the provided parameters.
    // let join_handler =
    //     tokio::spawn(
    //         async move { run(url.clone(), btc_string, tx, btc_websocket_keep_running).await },
    //     );
    // let xrp_handler = tokio::spawn(async move {
    //     run(
    //         xrp_url,
    //         xrp_string,
    //         xrp_channel_tx,
    //         xrp_websocket_keep_running,
    //     )
    //     .await
    // });

    // let bot_account_handler =
    //     tokio::spawn(
    //         async move { bot_account_run(&mut bot_signal_rx, bot_account_keep_running).await },
    //     );

    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal. Gracefully terminating...");

    keep_running.store(false, Ordering::SeqCst);
    // join_handler.abort();
    // // blocking_handler.abort();
    // xrp_handler.abort();
    // // trading_bot_handler.abort();
    // bot_account_handler.abort();

    Ok(())
}

fn setup_logging() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

fn launch_processing_tasks(
    mut rx: mpsc::Receiver<Vec<MarketTradeEvent>>,
    signal_tx: Sender<TradeSignal>,
    keep_running: Arc<AtomicBool>,
) {
    let (candle_tx, mut candle_rx) = mpsc::channel::<CandlestickMessage>(50);

    let candle_keep_running = keep_running.clone();
    let trading_bot_keep_running = keep_running.clone();

    spawn_blocking(move || candle(&mut rx, candle_tx, candle_keep_running));
    spawn_blocking(move || trading_bot_run(&mut candle_rx, signal_tx, trading_bot_keep_running));
}

fn launch_websocket_tasks(
    url: Url,
    tx: mpsc::Sender<Vec<MarketTradeEvent>>,
    mut signal_rx: mpsc::Receiver<TradeSignal>,
    keep_running: Arc<AtomicBool>,
) {
    let bot_account_keep_running = keep_running.clone();
    tokio::spawn(async move { bot_account_run(&mut signal_rx, bot_account_keep_running).await });

    let symbols = [BTC_SYMBOL, XRP_SYMBOL, ETH_SYMBOL, ADA_SYMBOL];
    for &symbol in symbols.iter() {
        let websocket_url = url.clone();
        let websocket_tx = tx.clone();
        let websocket_keep_running = keep_running.clone();
        let market_string = market_subcribe_string(symbol, USD_SYMBOL);

        tokio::spawn(async move {
            run(
                websocket_url,
                market_string,
                websocket_tx,
                websocket_keep_running,
            )
            .await
        });
    }
}

async fn run(
    ws_url: Url,
    market: String,
    tx: Sender<Vec<MarketTradeEvent>>,
    keep_running: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    while keep_running.load(Ordering::Relaxed) {
        match connect_async(ws_url.clone()).await {
            Ok((mut ws_stream, _)) => {
                info!("Connected to server!");

                subscribe(&mut ws_stream, &market, "subscribe").await;
                while let Some(msg) = ws_stream.next().await {
                    match msg {
                        Ok(event_msg) => match event_msg {
                            Message::Text(text) => {
                                let event: Event = serde_json::from_str(&text).unwrap();

                                match event {
                                    Event::Subscriptions(_) => {}
                                    Event::Heartbeats(heartbeat) => {
                                        info!("{:?}", heartbeat);
                                    }
                                    Event::MarketTrades(market_trades) => {
                                        // info!("{:?}", market_trades);
                                        let _ = tx.send(market_trades).await;
                                    }
                                }
                            }
                            Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => (),
                            Message::Close(e) => {
                                info!("Connection closed: {:?}", e)
                            }
                        },
                        Err(e) => {
                            error!("Error with websocket: {:?}", e)
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
    tx: Sender<CandlestickMessage>,
    keep_running: Arc<AtomicBool>,
) {
    let mut candlesticks = HashMap::new();
    candlesticks.insert(BTC_SYMBOL.to_string(), Candlestick::new(Utc::now()));
    candlesticks.insert(XRP_SYMBOL.to_string(), Candlestick::new(Utc::now()));
    candlesticks.insert(ETH_SYMBOL.to_string(), Candlestick::new(Utc::now()));
    candlesticks.insert(ADA_SYMBOL.to_string(), Candlestick::new(Utc::now()));

    while keep_running.load(Ordering::Relaxed) {
        while let Some(market_trades) = rx.blocking_recv() {
            for trade_event in market_trades.iter() {
                if trade_event.event_type == "snapshot" {
                    candle_snapshot(&mut candlesticks, &tx, &trade_event.trades);
                } else {
                    candle_update(&mut candlesticks, &tx, &trade_event.trades);
                }
            }
        }
    }
}

// #[inline]
// fn get_start_time(end_time: &DateTime<Utc>) -> DateTime<Utc> {
//     end_time.with_second(0).expect("Failed to set seconds to 0")
// }

#[inline]
fn get_start_time(end_time: &DateTime<Utc>) -> DateTime<Utc> {
    let seconds = end_time.second();
    let rounded_seconds = if seconds >= 1 { 1 } else { 0 };
    end_time
        .with_second(rounded_seconds)
        .expect("Failed to set seconds")
}

fn trading_bot_run(
    rx: &mut Receiver<CandlestickMessage>,
    signal_tx: Sender<TradeSignal>,
    keep_running: Arc<AtomicBool>,
) {
    let mut trading_bots = HashMap::new();
    trading_bots.insert(format!("{}-{}", BTC_SYMBOL, USD_SYMBOL), TradingBot::new());
    trading_bots.insert(format!("{}-{}", XRP_SYMBOL, USD_SYMBOL), TradingBot::new());
    trading_bots.insert(format!("{}-{}", ETH_SYMBOL, USD_SYMBOL), TradingBot::new());
    trading_bots.insert(format!("{}-{}", ADA_SYMBOL, USD_SYMBOL), TradingBot::new());

    while keep_running.load(Ordering::Relaxed) {
        if let Some(candlestick) = rx.blocking_recv() {
            if let Some(bot) = trading_bots.get_mut(&candlestick.symbol) {
                bot.update_bot(candlestick.candlestick);
                println!("TRADING BOT: {}", bot);
            }
            // signal = trading_bot.get_signal();

            // let _ = signal_tx.blocking_send(signal);
        }
    }
}

async fn bot_account_run(signal_rx: &mut Receiver<TradeSignal>, keep_running: Arc<AtomicBool>) {
    let mut bot_account = BotAccount::new();
    bot_account.update_balances().await;

    while keep_running.load(Ordering::Relaxed) {
        if let Some(signal) = signal_rx.recv().await {
            if signal == TradeSignal::Sell {
                // if signal == TradeSignal::Sell && bot_account.is_trade_active() {
                info!("SELL");
                // bot_account.create_order(model::TradeSide::Sell).await;
                // bot_account.update_balances().await;
            }
            // if signal == TradeSignal::Buy && !bot_account.is_trade_active() {

            if signal == TradeSignal::Buy {
                info!("SELL");

                // bot_account.create_order(model::TradeSide::Buy).await;
                // bot_account.update_balances().await;
            }
        }
    }
}
