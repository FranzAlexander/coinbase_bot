use std::{collections::HashMap, fmt};

use chrono::{DateTime, Duration, Timelike, Utc};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

use super::event::MarketTrade;

pub const CANDLESTICK_TIMEFRAME: i64 = 59;

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
    pub fn new() -> Self {
        Candlestick {
            start: Utc::now(),
            end: Utc::now(),
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

#[inline]
pub fn get_start_time(end_time: &DateTime<Utc>) -> DateTime<Utc> {
    end_time.with_second(0).expect("Failed to set seconds to 0")
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
