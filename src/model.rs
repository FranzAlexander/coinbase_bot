use chrono::{DateTime, NaiveDate, Utc};
use serde::{de::Error as DeError, Deserialize, Serialize};

// Common fields for all the messages
#[derive(Debug, Deserialize)]
pub struct CommonFields {
    pub channel: String,
    pub client_id: String,
    pub timestamp: DateTime<Utc>,
    pub sequence_num: u32,
}

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

// For the Subscriptions channel
#[derive(Debug, Deserialize)]
pub struct SubscriptionDetail {
    pub heartbeats: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionEvent {
    pub subscriptions: SubscriptionDetail,
}

// Event enum
#[derive(Debug)]
pub enum Event {
    Subscriptions(SubscriptionMessage),
    Heartbeats(HeartbeatMessage),
    Candles(CandlestickMessage),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarketTradesMessage {
    pub channel: String,
    pub client_id: String,
    pub timestamp: DateTime<Utc>,
    pub sequence_num: u32,
    pub events: Vec<MarketTradeEvent>,
}

#[derive(Debug, Deserialize)]
pub struct CandlestickMessage {
    channel: String,
    client_id: String,
    timestamp: String, // or use chrono::NaiveDateTime if you're using the `chrono` crate
    sequence_num: u64,
    pub events: Vec<CandlestickEvent>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarketTradeEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub trades: Vec<Trade>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize)]
pub struct CandlestickEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub candles: Vec<Candlestick>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Candlestick {
    #[serde(with = "string_or_i64")]
    pub start: i64,
    #[serde(with = "string_or_float")]
    pub high: f64,
    #[serde(with = "string_or_float")]
    pub low: f64,
    #[serde(with = "string_or_float")]
    pub open: f64,
    #[serde(with = "string_or_float")]
    pub close: f64,
    #[serde(with = "string_or_float")]
    pub volume: f64,
    pub product_id: String,
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
                    serde_json::from_value::<SubscriptionMessage>(v).map_err(DeError::custom)?;
                Ok(Event::Subscriptions(message))
            }
            Some("heartbeats") => {
                let message =
                    serde_json::from_value::<HeartbeatMessage>(v).map_err(DeError::custom)?;
                Ok(Event::Heartbeats(message))
            }
            Some("candles") => {
                let message =
                    serde_json::from_value::<CandlestickMessage>(v).map_err(DeError::custom)?;
                Ok(Event::Candles(message))
            }
            _ => Err(DeError::custom("Unknown channel")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountList {
    pub accounts: Vec<Account>,
    pub has_next: bool,
    pub cursor: Option<String>,
    pub size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub uuid: String,
    pub name: String,
    pub currency: String,
    pub available_balance: Balance,
    pub default: bool,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    #[serde(rename = "type")] // "type" is a keyword in Rust, so we rename it
    pub account_type: AccountType,
    pub ready: bool,
    pub hold: Balance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    #[serde(with = "string_or_float")]
    pub value: f64,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    AccountTypeUnspecified,
    AccountTypeCrypto,
    AccountTypeFiat,
    AccountTypeVault,
}
