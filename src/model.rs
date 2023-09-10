use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub enum Channel {
    Heartbeats,
    MarketTrades,
    L2Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Heartbeat {
    pub channel: String,
    client_id: String,
    timestamp: DateTime<Utc>,
    sequence_num: u32,
    events: Vec<HeartbeatEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatEvent {
    current_time: DateTime<Utc>,
    heartbeat_counter: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketTrade {
    pub channel: String,
    client_id: String,
    timestamp: DateTime<Utc>,
    sequence_num: u32,
    events: Vec<MarketTradeEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketTradeEvent {
    #[serde(rename = "type")]
    event_type: String,
    pub trades: Vec<Trade>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub product_id: String,
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub size: f64,
    pub side: TradeSide,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TradeSide {
    BUY,
    SELL,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeMessage {
    #[serde(rename = "type")]
    pub event_type: String,
    pub trades: Vec<Trade>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    current_time: DateTime<Utc>,
    heartbeat_counter: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventType {
    #[serde(rename = "trade_event")]
    TradeEvent(MarketTrade),
    #[serde(rename = "heartbeat")]
    Heartbeat(Heartbeat),
}

#[derive(Debug)]
pub struct OneMinuteCandle {
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
}

impl OneMinuteCandle {
    pub fn from_trades(trades: &[Trade]) -> Self {
        let open = trades.first().unwrap().price;
        let close = trades.last().unwrap().price;

        let high = trades
            .iter()
            .map(|trade| trade.price)
            .fold(f64::MIN, f64::max);
        let low = trades
            .iter()
            .map(|trade| trade.price)
            .fold(f64::MAX, f64::min);

        let volume = trades.iter().map(|trade| trade.size).sum(); // Compute total volume

        OneMinuteCandle {
            open,
            close,
            high,
            low,
            volume,
        }
    }
}

pub(crate) mod string_or_float {
    use std::fmt;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Float(f64),
        }

        match StringOrFloat::deserialize(deserializer)? {
            StringOrFloat::String(s) => {
                if s == "INF" {
                    Ok(f64::INFINITY)
                } else {
                    s.parse().map_err(de::Error::custom)
                }
            }
            StringOrFloat::Float(i) => Ok(i),
        }
    }
}
