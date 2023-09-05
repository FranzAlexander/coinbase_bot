use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub product_id: String,
    pub price: Decimal,
    pub size: Decimal,
    pub side: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Default, Clone)]
pub struct OneMinuteCandle {
    pub open: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub close: Option<Decimal>,
    pub volume: Decimal,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}
