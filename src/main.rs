use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use futures::StreamExt;
use model::Candlestick;
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

use crate::{
    account::BotAccount, model::MarketTradeEvent, trading_bot::TradingBot, util::subscribe,
};

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

    println!("Account: {:?}", bot_account);

    let trading_bot = Arc::new(Mutex::new(TradingBot::new()));

    let (tx, mut rx) = mpsc::channel::<Vec<MarketTradeEvent>>(250);

    let url = Url::parse("wss://advanced-trade-ws.coinbase.com")
        .context("Failed to create coinbase url")?;

    let candle_bot = trading_bot.clone();

    let candle_keep_going = Arc::new(AtomicBool::new(true));
    let candle_going = candle_keep_going.clone();
    let blocking_handler = spawn_blocking(move || candle(&mut rx, candle_bot, candle_going));

    let join_handler = tokio::spawn(async move { run(url, tx).await });

    tokio::signal::ctrl_c().await?;
    println!("Received shutdown signal. Gracefully terminating...");

    candle_keep_going.store(false, Ordering::SeqCst);
    join_handler.abort();
    blocking_handler.abort();

    Ok(())
}

async fn run(ws_url: Url, tx: Sender<Vec<MarketTradeEvent>>) -> anyhow::Result<()> {
    // let mut bot = TradingBot::new();

    loop {
        match connect_async(ws_url.clone()).await {
            Ok((mut ws_stream, _)) => {
                info!("Connected to server!");

                subscribe(&mut ws_stream, "XRP-USD", "subscribe").await;
                while let Some(msg) = ws_stream.next().await {
                    if let Ok(Message::Text(text)) = msg {
                        let event: model::Event =
                            serde_json::from_str(&text).context("failed to parse message")?;

                        match event {
                            model::Event::Subscriptions(_) => {}
                            model::Event::Heartbeats(_) => {}
                            model::Event::MarketTrades(market_trades) => {
                                info!("TRADE: {:?}", market_trades);
                                let _ = tx.send(market_trades.events).await;
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
}

fn candle(
    rx: &mut Receiver<Vec<MarketTradeEvent>>,
    trading_bot: Arc<Mutex<TradingBot>>,
    keep_running: Arc<AtomicBool>,
) {
    let mut candlestick: Candlestick = Candlestick::new(Utc::now());

    while keep_running.load(Ordering::Relaxed) {
        while let Some(x) = rx.blocking_recv() {
            if let Some(trade_event) = x.first() {
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
                            {
                                let mut locked_bot = trading_bot.blocking_lock();
                                locked_bot.update_bot(candlestick);
                                println!("{:?}, {:?}", locked_bot.short_ema, locked_bot.long_ema);
                            }
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
