use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use account::{get_product_candle, BotAccount, WS_URL};
use anyhow::bail;
use coin::CoinSymbol;
use futures::StreamExt;
use model::{
    event::{CandleEvent, CandleHistory, EventType},
    TradeSide,
};
use smallvec::SmallVec;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::spawn_blocking,
};
use tracing::{error, event, info, Level};
use trading_bot::{IndicatorGroup, IndicatorResult, TradeSignal, TradingBot};
use tungstenite::{connect, Message};
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

fn main() {
    let keep_running = Arc::new(AtomicBool::new(true));

    let symbols = [CoinSymbol::Xrp];
}

fn coin_trading_task(keep_running: Arc<AtomicBool>, symbol: CoinSymbol) {
    let mut trading_bot = TradingBot::new();
    let mut account_bot = BotAccount::new();
    account_bot.update_balances();

    let (mut socket, _) = connect(WS_URL).expect("Failed to connect to socket");

    while keep_running.load(Ordering::Relaxed) {
        let message = socket.read_message().unwrap();
        match message {
            Message::Text(msg) => {
                let event: Event = serde_json::from_str(&msg).unwrap();

                match event {
                    Event::Subscriptions(_) => (),
                    Event::Heartbeats(_) => (),
                    Event::Candle(candles) => {
                        let indicator_result = handle_candle(candles, &mut trading_bot, symbol);
                        if let Some(res) = indicator_result {}
                    }
                }
            }
            Message::Ping(_) => socket.write_message(Message::Pong(vec![])).unwrap(),
            Message::Binary(_) | Message::Pong(_) => (),
            Message::Close(e) => println!("Websocket closed: {:?}", e),
        }
    }
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

fn handle_candle(
    candles: SmallVec<[CandleEvent; 1]>,
    trading_bot: &mut TradingBot,
    symbol: CoinSymbol,
) -> Option<IndicatorResult> {
    for candle_event in candles.iter() {
        if candle_event.event_type == EventType::Snapshot && !trading_bot.initialise {
            let hist_candles =
                get_history_candles(symbol, candle_event.candles.last().unwrap().start);

            trading_bot.initialise = true;

            for hist_candle in hist_candles.candles.into_iter().rev() {
                trading_bot.one_minute_update(hist_candle);
            }

            for snap_candle in candle_event.candles.into_iter().rev() {
                trading_bot.one_minute_update(snap_candle);
            }
            return None;
        } else {
            for candle in candle_event.candles.into_iter() {
                if candle.start != trading_bot.start {
                    println!("Candle: {:?}", candle);
                    let signal = trading_bot.get_signal();
                    let atr = trading_bot.get_atr_value();
                    trading_bot.start = candle.start;

                    return Some(IndicatorResult {
                        signal,
                        atr,
                        high: candle.high,
                    });
                }
            }
        }
    }

    None
}

fn get_history_candles(symbol: CoinSymbol, recent_start: i64) -> CandleHistory {
    let end = recent_start - 300;
    let start = end - 30000;

    get_product_candle(symbol, start, end)
}

fn handle_signal(
    symbol: CoinSymbol,
    indicator_result: IndicatorResult,
    bot_account: &mut BotAccount,
) {
    println!("Current Signal: {:?}", indicator_result.signal);

    if bot_account.coin_trade_active(symbol) {
        let should_sell = bot_account.update_coin_position(
            symbol,
            indicator_result.high,
            indicator_result.atr.unwrap(),
        );

        // Add sell code here.
    }
    if !bot_account.coin_trade_active(symbol) && indicator_result.signal == TradeSignal::Buy {
        // Add buy code here.
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
