use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::{string_or_float, string_or_i64, TradeSide};

// Event enum
#[derive(Deserialize, Debug)]
#[serde(tag = "channel", content = "events")]
pub enum Event {
    #[serde(rename = "subscriptions")]
    Subscriptions(Vec<SubscriptionEvent>),
    #[serde(rename = "heartbeats")]
    Heartbeats(Vec<HeartbeatEvent>),
    #[serde(rename = "market_trades")]
    MarketTrades(Vec<MarketTradeEvent>),
    #[serde(rename = "ticker_batch")]
    Ticker(Vec<TickerEvent>),
}

#[derive(Deserialize, Debug)]
pub struct SubscriptionDetail {
    heartbeats: Option<Vec<String>>,
    market_trades: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct SubscriptionEvent {
    subscriptions: SubscriptionDetail,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatEvent {
    current_time: String, // or use chrono::NaiveDateTime
    heartbeat_counter: u64,
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
pub struct TickerEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub tickers: Vec<Ticker>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ticker {
    #[serde(rename = "type")]
    ticker_type: String,
    product_id: String,
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    volume_24_h: f64,
    #[serde(with = "string_or_float")]
    low_24_h: f64,
    #[serde(with = "string_or_float")]
    high_24_h: f64,
    #[serde(with = "string_or_float")]
    low_52_w: f64,
    #[serde(with = "string_or_float")]
    high_52_w: f64,
    #[serde(with = "string_or_float")]
    price_percent_chg_24_h: f64,
}
