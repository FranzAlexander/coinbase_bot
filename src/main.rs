use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use account::{BotAccount, WS_URL};
use candlestick::{get_start_time, Candlestick};
use chrono::Utc;
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
use trading_bot::{TradeSignal, TradingBot};
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
    position_open: Arc<Mutex<bool>>,
) {
    let mut trading_indicators: HashMap<CoinSymbol, (TradingBot, Candlestick)> = HashMap::new();
    let symbols = [CoinSymbol::Xrp];

    for symbol in symbols.into_iter() {
        trading_indicators.insert(
            symbol,
            (TradingBot::new(), Candlestick::new(Utc::now(), 0.0, 0.0)),
        );
    }

    while keep_running.load(Ordering::Relaxed) {
        while let Some(trades_msg) = indicator_rx.blocking_recv() {
            if let Some(indicator_bot) = trading_indicators.get_mut(&trades_msg.symbol) {
                for market_trades in trades_msg.trades {
                    // This runs when we first conncet to the websocket to get inital candle,
                    // the candle my not be finished in the snapshot.
                    if market_trades.event_type == EventType::Snapshot {
                        let first_trade_time = get_start_time(&market_trades.trades[0].time);
                        let valid_trades: Vec<&MarketTrade> = market_trades
                            .trades
                            .iter()
                            .filter(|&trade| trade.time >= first_trade_time)
                            .collect();

                        if let Some(&first_trade) = valid_trades.last() {
                            indicator_bot.1 = Candlestick::new(
                                first_trade.time,
                                first_trade.price,
                                first_trade.size,
                            );

                            for &trade in valid_trades.iter().rev().skip(1) {
                                indicator_bot.1.update(trade.price, trade.size);
                            }
                        }
                    } else {
                        for trade in market_trades.trades.iter().rev() {
                            {
                                let locked_position = position_open.blocking_lock();
                                if *locked_position {
                                    let atr = indicator_bot.0.get_atr_value();
                                    let _ = account_tx.blocking_send(AccountChannelMessage {
                                        timeframe: IndicatorTimeframe::PerTrade,
                                        symbol: trades_msg.symbol,
                                        start: 0,
                                        end: 0,
                                        signal: indicator_bot.0.get_rsi_signal(),
                                        high: trade.price,
                                        atr,
                                    });
                                }
                            }
                            if trade.time < indicator_bot.1.end {
                                indicator_bot.1.update(trade.price, trade.size);
                            } else {
                                indicator_bot.0.one_minute_update(indicator_bot.1);
                                println!("Candle: {}", indicator_bot.1);
                                println!("ATR: {:?}", indicator_bot.0.get_atr_value());
                                let signal =
                                    indicator_bot.0.get_signal(IndicatorTimeframe::OneMinute);
                                let atr = indicator_bot.0.get_atr_value();
                                let _ = account_tx.blocking_send(AccountChannelMessage {
                                    timeframe: IndicatorTimeframe::OneMinute,
                                    symbol: trades_msg.symbol,
                                    start: indicator_bot.1.start.timestamp(),
                                    end: indicator_bot.1.end.timestamp(),
                                    signal,
                                    high: indicator_bot.1.high,
                                    atr,
                                });
                                indicator_bot.1 =
                                    Candlestick::new(trade.time, trade.price, trade.size);
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
    position_open: Arc<Mutex<bool>>,
) {
    let mut bot_account = BotAccount::new();

    bot_account.update_balances().await;
    // bot_account.get_product_candle().await;

    while keep_running.load(Ordering::Relaxed) {
        while let Some(account_msg) = account_rx.recv().await {
            if account_msg.timeframe == IndicatorTimeframe::PerTrade
                && bot_account.coin_trade_active(account_msg.symbol)
            {
                let mut locked = position_open.lock().await;
                *locked = bot_account
                    .update_coin_position(
                        account_msg.symbol,
                        account_msg.high,
                        account_msg.atr.unwrap(),
                        account_msg.signal,
                    )
                    .await;
            }

            if account_msg.timeframe == IndicatorTimeframe::OneMinute
                && !bot_account.coin_trade_active(account_msg.symbol)
            {
                if account_msg.signal == TradeSignal::Buy {
                    bot_account
                        .create_order(TradeSide::Buy, account_msg.symbol, account_msg.atr.unwrap())
                        .await;
                    bot_account.update_balances().await;
                    bot_account.get_coin(account_msg.symbol);
                    {
                        let mut locked = position_open.lock().await;
                        *locked = true;
                    }
                }
            }
        }
    }
}
