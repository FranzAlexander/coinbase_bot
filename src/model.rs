use chrono::{DateTime, Timelike, Utc};
use serde::{de, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Bid,
    Offer,
}

// Event enum
#[derive(Debug)]
pub enum Event {
    Subscriptions(SubscriptionMessage),
    Heartbeats(HeartbeatMessage),
    MarketTrades(MarketTradeMessage),
}

// #[derive(Debug, Deserialize)]
// pub struct CoinbaseMessage {
//     channel: String,
//     client_id: String,
//     timestamp: DateTime<Utc>,
//     sequence_num: u32,
//     events: Vec<serde_json::Value>,
// }

#[derive(Debug, Deserialize)]
pub struct SubscriptionMessage {
    channel: String,
    client_id: String,
    timestamp: DateTime<Utc>,
    sequence_num: u32,
    events: Vec<serde_json::Value>, // Use a generic Value type for now.
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatMessage {
    channel: String,
    client_id: String,
    timestamp: String, // or use chrono::NaiveDateTime if you're using the `chrono` crate
    sequence_num: u64,
    events: Vec<HeartbeatEvent>,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatEvent {
    current_time: String, // or use chrono::NaiveDateTime
    heartbeat_counter: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarketTradeMessage {
    channel: String,
    client_id: String,
    timestamp: DateTime<Utc>, // or use chrono::NaiveDateTime if you're using the `chrono` crate
    sequence_num: u64,
    pub events: Vec<MarketTradeEvent>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarketTradeEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub trades: Vec<MarketTrade>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarketTrade {
    #[serde(with = "string_or_i64")]
    trade_id: i64,
    product_id: String,
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub size: f64,
    side: TradeSide,
    pub time: DateTime<Utc>,
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
        let end_time = start_time.with_minute(start_time.minute() + 1).unwrap();
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

pub(crate) mod string_or_i64 {
    use serde::{de, Deserialize, Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrI64 {
            String(String),
            Int(i64),
        }

        match StringOrI64::deserialize(deserializer)? {
            StringOrI64::String(s) => s.parse().map_err(de::Error::custom),
            StringOrI64::Int(i) => Ok(i),
        }
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: serde_json::Value = serde_json::Value::deserialize(deserializer)?;

        match v.get("channel").and_then(|c| c.as_str()) {
            Some("subscriptions") => {
                let message =
                    serde_json::from_value::<SubscriptionMessage>(v).map_err(de::Error::custom)?;
                Ok(Event::Subscriptions(message))
            }
            Some("heartbeats") => {
                let message =
                    serde_json::from_value::<HeartbeatMessage>(v).map_err(de::Error::custom)?;
                Ok(Event::Heartbeats(message))
            }
            Some("market_trades") => {
                let message =
                    serde_json::from_value::<MarketTradeMessage>(v).map_err(de::Error::custom)?;
                Ok(Event::MarketTrades(message))
            }

            _ => Err(de::Error::custom("Unknown channel")),
        }
    }
}
