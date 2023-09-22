use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

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
        // let end_time = start_time + Duration::seconds(30); // Change here
        let end_time = start_time + Duration::minutes(1);
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
}
