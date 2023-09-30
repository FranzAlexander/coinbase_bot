use std::{collections::HashMap, fmt};

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

use crate::get_start_time;

use super::event::MarketTrade;

pub const CANDLESTICK_TIMEFRAME: i64 = 5;

#[derive(Debug, Deserialize, Clone)]
pub struct CandlestickMessage {
    pub symbol: String,
    pub candlestick: Candlestick,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Candlestick {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub high: f64,
    pub low: Option<f64>,
    pub open: Option<f64>,
    pub close: f64,
    pub volume: f64,
}

impl Candlestick {
    pub fn new(start_time: DateTime<Utc>) -> Self {
        let end_time = start_time + Duration::minutes(CANDLESTICK_TIMEFRAME); // Change here

        Candlestick {
            start: start_time,
            end: end_time,
            high: 0.0,
            low: None,
            open: None,
            close: 0.0,
            volume: 0.0,
        }
    }

    pub fn update(&mut self, close: f64, volume: f64) {
        self.open.get_or_insert(close);
        self.high = self.high.max(close);
        self.low = Some(self.low.map_or(close, |l| l.min(close)));
        self.volume += volume;
        self.close = close;
    }

    pub fn reset(&mut self, start_time: DateTime<Utc>) {
        let end_time = start_time + Duration::seconds(CANDLESTICK_TIMEFRAME);
        self.start = start_time;
        self.end = end_time;
        self.high = 0.0;
        self.low = None;
        self.open = None;
        self.close = 0.0;
        self.volume = 0.0;
    }
}

pub fn candle_snapshot(
    candlesticks: &mut HashMap<String, Candlestick>,
    tx: &Sender<CandlestickMessage>,
    trades: &[MarketTrade],
) {
    if trades.is_empty() {
        return;
    }

    let symbol = &trades[0].product_id;
    let start_time = get_start_time(&trades[0].time);

    let candlestick = candlesticks
        .entry(symbol.to_string())
        .or_insert_with(|| Candlestick::new(start_time));
    candlestick.reset(start_time);

    for trade in trades.iter().rev() {
        if trade.time >= candlestick.start && trade.time < candlestick.end {
            candlestick.update(trade.price, trade.size);
        } else {
            // Close current candlestick and send it
            // println!("{} snapshot Candlestick: {}", symbol, candlestick);
            let _ = tx.blocking_send(CandlestickMessage {
                symbol: symbol.to_string(),
                candlestick: candlestick.clone(),
            });

            // Start a new candlestick
            let new_start_time = get_start_time(&trade.time);
            candlestick.reset(new_start_time);
            candlestick.update(trade.price, trade.size);
        }
    }
}

pub fn candle_update(
    candlesticks: &mut HashMap<String, Candlestick>,
    tx: &Sender<CandlestickMessage>,
    trades: &[MarketTrade],
) {
    if trades.is_empty() {
        return;
    }

    let symbol = &trades[0].product_id;

    // Ensure a candlestick exists for the symbol, and get a mutable reference to it.
    // If it doesn't exist yet, this could be an error or you might want to handle it differently.
    if let Some(candlestick) = candlesticks.get_mut(symbol) {
        for trade in trades.iter() {
            if trade.time >= candlestick.start && trade.time < candlestick.end {
                candlestick.update(trade.price, trade.size);
            } else {
                // Close current candlestick and send it
                // println!("{} update Candlestick: {}", symbol, candlestick);
                let _ = tx.blocking_send(CandlestickMessage {
                    symbol: symbol.to_string(),
                    candlestick: candlestick.clone(),
                });

                // Start a new candlestick
                let new_start_time = get_start_time(&trade.time);
                candlestick.reset(new_start_time);
                candlestick.update(trade.price, trade.size);
            }
        }
    } else {
        // Handle cases where the symbol isn't found. Maybe log it, or handle it according to your needs.
        eprintln!("Received trade update for unknown symbol: {}", symbol);
    }
}

impl fmt::Display for Candlestick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Start: {}, End: {}, Open: {}, High: {}, Low: {}, Close: {}, Volume: {}",
            self.start,
            self.end,
            self.open.unwrap_or(0.0),
            self.high,
            self.low.unwrap_or(0.0),
            self.close,
            self.volume
        )
    }
}
