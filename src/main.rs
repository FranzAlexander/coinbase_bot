use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use account::{get_product_candle, BotAccount, WS_URL};
use coin::CoinSymbol;
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

    let symbols = [
        CoinSymbol::Xrp,
        CoinSymbol::Btc,
        CoinSymbol::Eth,
        CoinSymbol::Link,
        CoinSymbol::Ltc,
    ];
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();

    let num_symbols = symbols.len();

    for symbol in symbols.into_iter() {
        let coin_keep_running = keep_running.clone();
        let handle =
            thread::spawn(move || coin_trading_task(coin_keep_running, symbol, num_symbols));
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn coin_trading_task(keep_running: Arc<AtomicBool>, symbol: CoinSymbol, num_symbols: usize) {
    let mut trading_bot = TradingBot::new();
    let mut account_bot = BotAccount::new(num_symbols);
    account_bot.update_balances(symbol);

    let (mut socket, _) = connect(WS_URL).expect("Failed to connect to socket");

    println!("Connected to server!");

    let market_string =
        market_subcribe_string(&String::from(symbol), &String::from(CoinSymbol::Usdc));

    subscribe(&mut socket, &market_string, "subscribe");

    let mut backoff_time = 1;

    while keep_running.load(Ordering::Relaxed) {
        match socket.read_message() {
            Ok(message) => match message {
                Message::Text(msg) => {
                    backoff_time = 1;
                    let event: Event = serde_json::from_str(&msg).unwrap();

                    match event {
                        Event::Subscriptions(_) => (),
                        Event::Heartbeats(_) => (),
                        Event::Candle(candles) => {
                            let indicator_result = handle_candle(candles, &mut trading_bot, symbol);
                            if let Some(res) = indicator_result {
                                println!("CAN TRADE: {}", trading_bot.get_can_trade());
                                handle_signal(symbol, res, &mut account_bot, &mut trading_bot);
                            }
                        }
                    }
                }
                Message::Ping(_) => socket.write_message(Message::Pong(vec![])).unwrap(),
                Message::Binary(_) | Message::Pong(_) => (),
                Message::Close(e) => println!("Websocket closed: {:?}", e),
            },
            Err(_) => {
                println!(
                    "Connection lost. Reconnecting in {} seconds...",
                    backoff_time
                );
                std::thread::sleep(std::time::Duration::from_secs(backoff_time));

                backoff_time = (backoff_time * 2).min(60); // Double the backoff time, but cap it at 60 seconds
                let connection_result = connect(WS_URL);
                if let Ok((new_socket, _)) = connection_result {
                    socket = new_socket;
                    println!("Successfully reconnected!");

                    // Re-subscribe after reconnecting
                    let market_string = market_subcribe_string(
                        &String::from(symbol),
                        &String::from(CoinSymbol::Usdc),
                    );
                    subscribe(&mut socket, &market_string, "subscribe");
                } else {
                    println!("Failed to reconnect. Will try again...");
                }
            }
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
        }
        if candle_event.event_type == EventType::Update {
            for candle in candle_event.candles.iter() {
                if candle.start != trading_bot.candle.start {
                    println!("{:?}", trading_bot.candle);
                    trading_bot.one_minute_update(trading_bot.candle);
                    let signal = trading_bot.get_signal(trading_bot.candle.close);
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
    trading_bot: &mut TradingBot,
) {
    println!("Current Signal: {:?}", indicator_result.signal);

    if !bot_account.can_trade() {
        let should_sell =
            bot_account.update_coin_position(indicator_result.high, indicator_result.atr.unwrap());

        if should_sell {
            println!("Closing Open Position");
            bot_account.create_order(
                TradeSide::Sell,
                symbol,
                indicator_result.atr.unwrap(),
                indicator_result.high,
            );
            bot_account.update_balances(symbol);
        }
    }
    if bot_account.can_trade()
        && indicator_result.signal == TradeSignal::Buy
        && trading_bot.get_can_trade()
    {
        println!("Entering Open Position");
        bot_account.create_order(
            TradeSide::Buy,
            symbol,
            indicator_result.atr.unwrap(),
            indicator_result.high,
        );
        trading_bot.set_can_trade(false);
        bot_account.update_balances(symbol);
    }
}
