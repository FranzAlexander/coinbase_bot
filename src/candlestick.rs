use std::fmt;

use chrono::{DateTime, Duration, Timelike, Utc};
use serde::Deserialize;

use crate::trading_bot::IndicatorTimeframe;

pub const CANDLESTICK_ONE_MIN_TIMEFRAME: i64 = 60;
pub const CANDLESTICK_FIVE_MIN_TIMEFRAME: i64 = 300;

#[derive(Debug, Deserialize, Clone)]
pub struct CandlestickMessage {
    pub symbol: String,
    pub candlestick: Candlestick,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Candlestick {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candlestick {
    pub fn new(time: DateTime<Utc>, price: f64, size: f64, timeframe: IndicatorTimeframe) -> Self {
        let start = get_start_time(&time);
        let mut time_f = 0;
        if timeframe == IndicatorTimeframe::OneMinute {
            time_f = CANDLESTICK_ONE_MIN_TIMEFRAME;
        }
        if timeframe == IndicatorTimeframe::FiveMinute {
            time_f = CANDLESTICK_FIVE_MIN_TIMEFRAME;
        }
        let end = start + Duration::seconds(time_f);
        Candlestick {
            start,
            end,
            high: price,
            low: price,
            open: price,
            close: price,
            volume: size,
        }
    }

    pub fn update(&mut self, price: f64, size: f64) {
        if price > self.high {
            self.high = price;
        }
        if price < self.low {
            self.low = price;
        }
        self.close = price;
        self.volume += size;
    }
}

#[inline]
pub fn get_start_time(time: &DateTime<Utc>) -> DateTime<Utc> {
    time.with_second(0)
        .expect("Failed to set seconds to 0")
        .with_nanosecond(0)
        .unwrap()
}

impl fmt::Display for Candlestick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Start: {}, Low: {}, High: {}, Open: {}, Close: {}, Volume: {}",
            self.start, self.low, self.high, self.open, self.close, self.volume
        )
    }
}
