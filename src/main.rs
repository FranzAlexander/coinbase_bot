use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use coin::CoinSymbol;
use futures::{SinkExt, StreamExt};
use model::{
    account::ActiveTrade,
    candlestick::{candle_snapshot, candle_update, Candlestick, CandlestickMessage},
    event::{MarketTradeEvent, PriceUpdates},
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
use util::insert_into_hashmap;

use crate::{
    account::BotAccount,
    model::{channel::AccountChannelMessage, event::Event},
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

    let (account_channel_tx, mut account_channel_rx) = mpsc::channel::<AccountChannelMessage>(100);

    // Parse the WebSocket URL for the Coinbase exchange.
    let url = Url::parse("wss://advanced-trade-ws.coinbase.com")
        .context("Failed to create coinbase url")?;

    // Create a shared AtomicBool flag to control when to keep running various components.
    let keep_running = Arc::new(AtomicBool::new(true));

    // Clone the keep_running flag for use in the WebSocket component.
    let candle_going = keep_running.clone();
    let trading_bot_keep_going = keep_running.clone();

    launch_processing_tasks(rx, account_channel_tx.clone(), keep_running.clone());
    launch_websocket_tasks(url, tx, account_channel_rx, keep_running.clone());

    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal. Gracefully terminating...");

    keep_running.store(false, Ordering::SeqCst);

    Ok(())
}

fn setup_logging() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

fn launch_processing_tasks(
    mut rx: mpsc::Receiver<Vec<MarketTradeEvent>>,
    account_channel_tx: Sender<AccountChannelMessage>,
    keep_running: Arc<AtomicBool>,
) {
    let (candle_tx, mut candle_rx) = mpsc::channel::<CandlestickMessage>(50);

    let candle_keep_running = keep_running.clone();
    let trading_bot_keep_running = keep_running.clone();

    spawn_blocking(move || candle(&mut rx, candle_tx, candle_keep_running));
    spawn_blocking(move || {
        trading_bot_run(&mut candle_rx, account_channel_tx, trading_bot_keep_running)
    });
}

fn launch_websocket_tasks(
    url: Url,
    tx: mpsc::Sender<Vec<MarketTradeEvent>>,
    mut account_channel_rx: mpsc::Receiver<AccountChannelMessage>,
    keep_running: Arc<AtomicBool>,
) {
    let price_updates = Arc::new(Mutex::new(PriceUpdates {
        send: false,
        symbol: "".to_string(),
    }));
    let bot_account_keep_running = keep_running.clone();
    let bot_price_update = price_updates.clone();

    let (price_tx, mut price_rx) = mpsc::channel::<f64>(10);
    tokio::spawn(async move {
        bot_account_run(
            &mut account_channel_rx,
            bot_account_keep_running,
            bot_price_update,
        )
        .await
    });

    let symbols = [CoinSymbol::Xrp, CoinSymbol::Link];
    for symbol in symbols.into_iter() {
        let websocket_url = url.clone();
        let websocket_tx = tx.clone();
        let websocket_keep_running = keep_running.clone();
        let market_string =
            market_subcribe_string(&String::from(symbol), &String::from(CoinSymbol::Usd));
        let price_update = price_updates.clone();
        let websocket_price_tx = price_tx.clone();
        tokio::spawn(async move {
            run(
                websocket_url,
                market_string,
                websocket_tx,
                websocket_keep_running,
                price_update,
                websocket_price_tx,
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
    price_update: Arc<Mutex<PriceUpdates>>,
    price_tx: Sender<f64>,
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
                                        let _ = tx.send(market_trades.clone()).await;
                                        {
                                            let locked_price = price_update.lock().await;
                                            if locked_price.send && locked_price.symbol == market {
                                                let price = market_trades
                                                    .last()
                                                    .unwrap()
                                                    .trades
                                                    .last()
                                                    .unwrap()
                                                    .price;
                                                let _ = price_tx.send(price).await;
                                            }
                                        }
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
    candlesticks.insert(String::from(CoinSymbol::Xrp), Candlestick::new(Utc::now()));
    candlesticks.insert(String::from(CoinSymbol::Ada), Candlestick::new(Utc::now()));
    candlesticks.insert(String::from(CoinSymbol::Link), Candlestick::new(Utc::now()));

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

#[inline]
fn get_start_time(end_time: &DateTime<Utc>) -> DateTime<Utc> {
    end_time.with_second(0).expect("Failed to set seconds to 0")
}

fn trading_bot_run(
    rx: &mut Receiver<CandlestickMessage>,
    account_channel_tx: Sender<AccountChannelMessage>,
    keep_running: Arc<AtomicBool>,
) {
    let mut trading_bots = HashMap::new();
    insert_into_hashmap::<TradingBot>(
        &mut trading_bots,
        CoinSymbol::Xrp,
        CoinSymbol::Usd,
        TradingBot::new(),
    );
    insert_into_hashmap::<TradingBot>(
        &mut trading_bots,
        CoinSymbol::Ada,
        CoinSymbol::Usd,
        TradingBot::new(),
    );
    insert_into_hashmap::<TradingBot>(
        &mut trading_bots,
        CoinSymbol::Link,
        CoinSymbol::Usd,
        TradingBot::new(),
    );

    while keep_running.load(Ordering::Relaxed) {
        if let Some(candlestick) = rx.blocking_recv() {
            if let Some(bot) = trading_bots.get_mut(&candlestick.symbol) {
                bot.update_bot(candlestick.candlestick);
            }
        }
    }
}

async fn bot_account_run(
    rx: &mut Receiver<AccountChannelMessage>,
    keep_running: Arc<AtomicBool>,
    price_update: Arc<Mutex<PriceUpdates>>,
) {
    let mut bot_account = BotAccount::new();
    bot_account.update_balances().await;
    bot_account.get_transaction_summary().await;

    while keep_running.load(Ordering::Relaxed) {
        if let Some(message) = rx.recv().await {
            match message.symbol {
                coin::CoinSymbol::Xrp => {
                    bot_account
                        .handle_message((message.signal, message.price))
                        .await;
                }
                coin::CoinSymbol::Ada => {
                    bot_account
                        .handle_message((message.signal, message.price))
                        .await;
                }
                coin::CoinSymbol::Link => {
                    bot_account
                        .handle_message((message.signal, message.price))
                        .await;
                }
                coin::CoinSymbol::Usd
                | coin::CoinSymbol::Usdc
                | coin::CoinSymbol::Btc
                | coin::CoinSymbol::Eth => (),
                CoinSymbol::Unknown => (),
            }
            // if signal == TradeSignal::Sell {
            //     // if signal == TradeSignal::Sell && bot_account.is_trade_active() {
            //     info!("SELL");
            //     // bot_account.create_order(model::TradeSide::Sell).await;
            //     // bot_account.update_balances().await;
            // }
            // // if signal == TradeSignal::Buy && !bot_account.is_trade_active() {

            // if signal == TradeSignal::Buy {
            //     info!("SELL");

            //     // bot_account.create_order(model::TradeSide::Buy).await;
            //     // bot_account.update_balances().await;
            // }
        }
        info!("PRINTT");
    }
}
