use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use account::{get_product_candle, BotAccount, WS_URL};
use candlestick::{get_start_time, Candlestick};
use chrono::{Duration, Utc};
use coin::CoinSymbol;
use futures::StreamExt;
use model::{
    event::{EventType, MarketTrade},
    TradeSide,
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
use trading_bot::{IndicatorGroup, TradeSignal, TradingBot};
use util::market_subcribe_string;

use crate::{
    model::{
        channel::{AccountChannelMessage, IndicatorChannelMessage},
        event::Event,
    },
    trading_bot::IndicatorTimeframe,
    util::subscribe,
};

mod account;
mod candlestick;
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
                                    Event::MarketTrades(market_trades) => {
                                        let _ = indicator_tx
                                            .send(IndicatorChannelMessage {
                                                symbol,
                                                trades: market_trades,
                                            })
                                            .await;
                                    }
                                    Event::Candle(candle) => {
                                        info!("{:?}", candle);
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
    let symbols = [(CoinSymbol::Xrp, IndicatorTimeframe::OneMinute)];

    for symbol in symbols.into_iter() {
        trading_indicators.insert(
            symbol.0,
            IndicatorGroup {
                timeframe: symbol.1,
                trading_bot: TradingBot::new(),
                candle: Candlestick::new(Utc::now(), 0.0, 0.0, symbol.1),
                initialise: true,
            },
        );
    }

    while keep_running.load(Ordering::Relaxed) {
        if let Some(trades_msg) = indicator_rx.blocking_recv() {
            if let Some(indicator_bot) = trading_indicators.get_mut(&trades_msg.symbol) {
                for market_trades in trades_msg.trades {
                    // This runs when we first conncet to the websocket to get inital candle,
                    // the candle my not be finished in the snapshot.
                    if market_trades.event_type == EventType::Snapshot && indicator_bot.initialise {
                        let first_trade_time = get_start_time(&market_trades.trades[0].time);
                        let valid_trades: Vec<&MarketTrade> = market_trades
                            .trades
                            .iter()
                            .filter(|&trade| trade.time >= first_trade_time)
                            .collect();

                        if let Some(&first_trade) = valid_trades.last() {
                            let end = first_trade.time;
                            let start = first_trade.time - Duration::minutes(100);
                            let his_candles = get_product_candle(
                                trades_msg.symbol,
                                start.timestamp(),
                                end.timestamp(),
                                indicator_bot.timeframe,
                            );

                            for candle in his_candles.iter().rev() {
                                indicator_bot
                                    .trading_bot
                                    .one_minute_update(Candlestick::new(
                                        Utc::now(),
                                        candle.close,
                                        candle.volume,
                                        indicator_bot.timeframe,
                                    ));
                            }
                            indicator_bot.candle = Candlestick::new(
                                first_trade.time,
                                first_trade.price,
                                first_trade.size,
                                indicator_bot.timeframe,
                            );

                            for &trade in valid_trades.iter().rev().skip(1) {
                                indicator_bot.candle.update(trade.price, trade.size);
                            }

                            indicator_bot.initialise = false;
                        }
                    } else {
                        for trade in market_trades.trades.iter().rev() {
                            if trade.time < indicator_bot.candle.end {
                                indicator_bot.candle.update(trade.price, trade.size);
                            } else {
                                indicator_bot
                                    .trading_bot
                                    .one_minute_update(indicator_bot.candle);
                                println!("Candle: {}", indicator_bot.candle);
                                let signal = indicator_bot
                                    .trading_bot
                                    .get_signal(IndicatorTimeframe::OneMinute);
                                let atr = indicator_bot.trading_bot.get_atr_value();
                                let _ = account_tx.blocking_send(AccountChannelMessage {
                                    timeframe: indicator_bot.timeframe,
                                    symbol: trades_msg.symbol,
                                    start: indicator_bot.candle.start.timestamp(),
                                    end: indicator_bot.candle.end.timestamp(),
                                    signal,
                                    high: indicator_bot.candle.high,
                                    atr,
                                });
                                indicator_bot.candle = Candlestick::new(
                                    trade.time,
                                    trade.price,
                                    trade.size,
                                    indicator_bot.timeframe,
                                );
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
            if account_msg.timeframe == IndicatorTimeframe::OneMinute
                && !bot_account.coin_trade_active(account_msg.symbol).await
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
