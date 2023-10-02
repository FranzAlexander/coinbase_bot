use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use account::{BotAccount, WS_URL};
use coin::CoinSymbol;
use futures::StreamExt;
use model::{
    candlestick::{get_start_time, Candlestick},
    event::EventType,
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
use trading_bot::TradingBot;
use util::market_subcribe_string;

use crate::{
    model::{channel::IndicatorChannelMessage, event::Event},
    trading_bot::IndicatorTimeframe,
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

    let keep_running = Arc::new(AtomicBool::new(true));

    launch_processing_tasks(indicator_rx, keep_running.clone());
    launch_websocket_tasks(indicator_tx, keep_running.clone());

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
    keep_running: Arc<AtomicBool>,
) {
    let bot_account = Arc::new(Mutex::new(BotAccount::new()));
    let symbols = [CoinSymbol::Xrp, CoinSymbol::Link];

    for symbol in symbols.into_iter() {
        let websocket_keep_running = keep_running.clone();
        let market_string = market_subcribe_string(
            &String::from(symbol.clone()),
            &String::from(CoinSymbol::Usd),
        );
        let websocket_bot = bot_account.clone();
        let websocket_tx = indicator_tx.clone();
        tokio::spawn(async move {
            run_websocket(
                symbol,
                market_string,
                websocket_keep_running,
                websocket_bot,
                websocket_tx,
            )
            .await
        });
    }
}

fn launch_processing_tasks(
    mut indicator_rx: Receiver<IndicatorChannelMessage>,
    keep_running: Arc<AtomicBool>,
) {
    spawn_blocking(move || run_indicator(&mut indicator_rx, keep_running));
}

async fn run_websocket(
    symbol: CoinSymbol,
    market: String,
    keep_running: Arc<AtomicBool>,
    bot_account: Arc<Mutex<BotAccount>>,
    indicator_tx: Sender<IndicatorChannelMessage>,
) {
    while keep_running.load(Ordering::Relaxed) {
        match connect_async(WS_URL).await {
            Ok((mut ws_stream, _)) => {
                info!("Connected to server!");

                {
                    let locked_account = bot_account.lock().await;
                    subscribe(
                        &mut ws_stream,
                        &market,
                        "subscribe",
                        locked_account.get_api_key(),
                    )
                    .await;
                }

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
                                                timeframe: IndicatorTimeframe::PerTrade,
                                                symbol: symbol.clone(),
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
    keep_running: Arc<AtomicBool>,
) {
    let mut trading_indicators: HashMap<CoinSymbol, (TradingBot, Candlestick)> = HashMap::new();
    let symbols = [CoinSymbol::Xrp, CoinSymbol::Link];

    for symbol in symbols.into_iter() {
        trading_indicators.insert(symbol, (TradingBot::new(), Candlestick::new()));
    }

    while keep_running.load(Ordering::Relaxed) {
        while let Some(trades_msg) = indicator_rx.blocking_recv() {
            if let Some(indicator_bot) = trading_indicators.get_mut(&trades_msg.symbol) {
                for market_trades in trades_msg.trades {
                    if market_trades.event_type == EventType::Snapshot {
                        indicator_bot
                            .1
                            .reset(get_start_time(&market_trades.trades.last().unwrap().time));
                    }

                    for trade in market_trades.trades.iter().rev() {
                        indicator_bot.0.per_trade_update(trade.clone());

                        if trade.time >= indicator_bot.1.start && trade.time < indicator_bot.1.end {
                            indicator_bot.1.update(trade.price, trade.size);
                        } else {
                            indicator_bot.0.one_minute_update(indicator_bot.1.clone());
                            indicator_bot.1.reset(get_start_time(&trade.time));
                        }
                    }
                    println!("BOT: {:?}", indicator_bot.0);
                    println!(
                        "{:?}",
                        indicator_bot.0.get_signal(trades_msg.timeframe.clone())
                    );
                }
            } else {
                println!("Symbol not found");
            }
        }
    }
}
