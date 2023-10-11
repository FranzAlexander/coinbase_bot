use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
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

use trading_bot::{IndicatorResult, TradeSignal, TradingBot};
use tungstenite::{connect, Message};
use util::{market_subcribe_string, subscribe};

use crate::model::event::Event;

mod account;
mod coin;
mod indicators;
mod model;
mod trading_bot;
mod util;

fn main() {
    let keep_running = Arc::new(AtomicBool::new(true));

    let symbols = [CoinSymbol::Xrp, CoinSymbol::Btc];
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();

    for symbol in symbols.into_iter() {
        let coin_keep_running = keep_running.clone();
        let handle = thread::spawn(move || coin_trading_task(coin_keep_running, symbol));
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn coin_trading_task(keep_running: Arc<AtomicBool>, symbol: CoinSymbol) {
    let mut trading_bot = TradingBot::new();
    let mut account_bot = BotAccount::new();
    account_bot.update_balances(symbol);

    let (mut socket, _) = connect(WS_URL).expect("Failed to connect to socket");

    let market_string =
        market_subcribe_string(&String::from(symbol), &String::from(CoinSymbol::Usdc));

    subscribe(&mut socket, &market_string, "subscribe");

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
                        if let Some(res) = indicator_result {
                            handle_signal(
                                symbol,
                                res,
                                &mut account_bot,
                                trading_bot.get_can_trade(),
                            );
                        }
                    }
                }
            }
            Message::Ping(_) => socket.write_message(Message::Pong(vec![])).unwrap(),
            Message::Binary(_) | Message::Pong(_) => (),
            Message::Close(e) => println!("Websocket closed: {:?}", e),
        }
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

            for hist_candle in hist_candles.candles.iter().rev() {
                trading_bot.one_minute_update(*hist_candle);
            }

            for snap_candle in candle_event.candles.iter() {
                trading_bot.one_minute_update(*snap_candle);
                trading_bot.candle = *snap_candle;
            }
            return None;
        } else {
            for candle in candle_event.candles.iter() {
                if candle.start != trading_bot.candle.start {
                    trading_bot.one_minute_update(trading_bot.candle);
                    let signal = trading_bot.get_signal();
                    let atr = trading_bot.get_atr_value();
                    trading_bot.candle = *candle;

                    return Some(IndicatorResult {
                        signal,
                        atr,
                        high: candle.high,
                    });
                } else {
                    trading_bot.candle = *candle;
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
    trader_can_trade: bool,
) {
    println!("Current Signal: {:?}", indicator_result.signal);

    if !bot_account.can_trade() {
        let should_sell =
            bot_account.update_coin_position(indicator_result.high, indicator_result.atr.unwrap());

        if should_sell {
            bot_account.create_order(TradeSide::Sell, symbol, indicator_result.atr.unwrap());
            bot_account.update_balances(symbol);
        }
    }
    if bot_account.can_trade() && indicator_result.signal == TradeSignal::Buy && trader_can_trade {
        bot_account.create_order(TradeSide::Buy, symbol, indicator_result.atr.unwrap());
        bot_account.update_balances(symbol);
    }
}
