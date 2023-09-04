use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    trade_id: String,
    product_id: String,
    price: Decimal,
    size: Decimal,
    side: String,
    time: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct OneMinuteCandle {
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    start_time: DateTime<Utc>,
}
