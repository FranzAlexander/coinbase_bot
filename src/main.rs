use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use account::{get_product_candle, BotAccount, WS_URL};
use chrono::{Duration, Utc};
use coin::CoinSymbol;
use futures::StreamExt;
use model::{event::EventType, TradeSide};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::spawn_blocking,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, event, info, Level};
use trading_bot::{IndicatorGroup, TradeSignal, TradingBot};
use util::market_subcribe_string;

use crate::{
    model::{
        channel::{AccountChannelMessage, IndicatorChannelMessage},
        event::Event,
    },
    util::subscribe,
};

mod account;
mod coin;
mod indicators;
mod model;
mod trading_bot;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging()?;

    let (indicator_tx, indicator_rx) = mpsc::channel::<IndicatorChannelMessage>(200);
    let (account_tx, account_rx) = mpsc::channel::<AccountChannelMessage>(200);

    let position_open = Arc::new(Mutex::new(false));
    let keep_running = Arc::new(AtomicBool::new(true));

    launch_processing_tasks(
        indicator_rx,
        account_tx.clone(),
        keep_running.clone(),
        position_open.clone(),
    );
    launch_websocket_tasks(
        indicator_tx,
        account_rx,
        keep_running.clone(),
        position_open.clone(),
    );

    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal. Gracefully terminating...");

    keep_running.store(false, Ordering::SeqCst);

    Ok(())
}

fn setup_logging() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

fn launch_websocket_tasks(
    indicator_tx: Sender<IndicatorChannelMessage>,
    mut account_rx: Receiver<AccountChannelMessage>,
    keep_running: Arc<AtomicBool>,
    position_open: Arc<Mutex<bool>>,
) {
    let bot_account_keep_running = keep_running.clone();

    tokio::spawn(async move {
        run_bot_account(&mut account_rx, bot_account_keep_running, position_open).await
    });

    let symbols = [CoinSymbol::Xrp];

    for symbol in symbols.into_iter() {
        let websocket_keep_running = keep_running.clone();
        let market_string =
            market_subcribe_string(&String::from(symbol), &String::from(CoinSymbol::Usd));
        let websocket_tx = indicator_tx.clone();

        tokio::spawn(async move {
            run_websocket(symbol, market_string, websocket_keep_running, websocket_tx).await
        });
    }
}

fn launch_processing_tasks(
    mut indicator_rx: Receiver<IndicatorChannelMessage>,
    account_tx: Sender<AccountChannelMessage>,
    keep_running: Arc<AtomicBool>,
    position_open: Arc<Mutex<bool>>,
) {
    spawn_blocking(move || {
        run_indicator(&mut indicator_rx, account_tx, keep_running, position_open)
    });
}

async fn run_websocket(
    symbol: CoinSymbol,
    market: String,
    keep_running: Arc<AtomicBool>,
    indicator_tx: Sender<IndicatorChannelMessage>,
) {
    while keep_running.load(Ordering::Relaxed) {
        match connect_async(WS_URL).await {
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
                                    // Event::MarketTrades(market_trades) => {
                                    //     let _ = indicator_tx
                                    //         .send(IndicatorChannelMessage {
                                    //             symbol,
                                    //             trades: market_trades,
                                    //         })
                                    //         .await;
                                    // }
                                    Event::Candle(candle) => {
                                        let _ = indicator_tx
                                            .send(IndicatorChannelMessage {
                                                symbol,
                                                candles: candle,
                                            })
                                            .await;
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
            }
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Failed to connect to {}: {}. Retrying in {} seconds...",
                    WS_URL,
                    e,
                    3
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }
    }
}

fn run_indicator(
    indicator_rx: &mut Receiver<IndicatorChannelMessage>,
    account_tx: Sender<AccountChannelMessage>,
    keep_running: Arc<AtomicBool>,
    _position_open: Arc<Mutex<bool>>,
) {
    let mut trading_indicators: HashMap<CoinSymbol, IndicatorGroup> = HashMap::new();
    let symbols = [CoinSymbol::Xrp];

    for symbol in symbols.into_iter() {
        trading_indicators.insert(
            symbol,
            IndicatorGroup {
                trading_bot: TradingBot::new(),
                start: 0,
                initialise: true,
            },
        );
    }

    while keep_running.load(Ordering::Relaxed) {
        if let Some(message) = indicator_rx.blocking_recv() {
            if let Some(indicator_bot) = trading_indicators.get_mut(&message.symbol) {
                for candle_event in message.candles.iter() {
                    if candle_event.event_type == EventType::Snapshot && indicator_bot.initialise {
                        let end = candle_event
                            .candles
                            .last()
                            .expect("Failed to get last candle")
                            .start;

                        let start = end - 6000;

                        let candles = get_product_candle(message.symbol, start, end);

                        indicator_bot.initialise = false;
                        println!("{:?}", candles);
                    } else {
                        for candle in candle_event.candles.iter().rev() {
                            if candle.start != indicator_bot.start {
                                indicator_bot.trading_bot.one_minute_update(candle.clone());
                                let signal = indicator_bot.trading_bot.get_signal();
                                let atr = indicator_bot.trading_bot.get_atr_value();
                                indicator_bot.start = candle.start;
                                let _ = account_tx.blocking_send(AccountChannelMessage {
                                    symbol: message.symbol,
                                    signal,
                                    atr,
                                    high: candle.high,
                                });
                            }
                        }
                    }
                }
            } else {
                println!("Symbol not found");
            }
        }
    }
}

async fn run_bot_account(
    account_rx: &mut Receiver<AccountChannelMessage>,
    keep_running: Arc<AtomicBool>,
    _position_open: Arc<Mutex<bool>>,
) {
    let mut bot_account = BotAccount::new();

    bot_account.update_balances().await;

    while keep_running.load(Ordering::Relaxed) {
        while let Some(account_msg) = account_rx.recv().await {
            info!("CURRENT SIGNAL: {:?}", account_msg.signal);
            if bot_account.coin_trade_active(account_msg.symbol).await {
                let sell = bot_account
                    .update_coin_position(
                        account_msg.symbol,
                        account_msg.high,
                        account_msg.atr.unwrap(),
                    )
                    .await;

                if sell {
                    bot_account
                        .create_order(
                            TradeSide::Sell,
                            account_msg.symbol,
                            account_msg.atr.unwrap(),
                        )
                        .await;
                    bot_account.update_balances().await;
                }
            }
            if !bot_account.coin_trade_active(account_msg.symbol).await
                && account_msg.signal == TradeSignal::Buy
            {
                bot_account
                    .create_order(TradeSide::Buy, account_msg.symbol, account_msg.atr.unwrap())
                    .await;
                bot_account.update_balances().await;
            }
        }
    }
}
