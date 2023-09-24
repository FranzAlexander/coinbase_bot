use std::fmt;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use super::{string_or_float, string_or_i64, OrderStatus, TradeSide};

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
    current_time: DateTime<Utc>,
    heartbeat_counter: u64,
}

impl fmt::Display for HeartbeatEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Current Time: {}, Heartbeat Counter: {}",
            self.current_time, self.heartbeat_counter
        )
    }
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
pub struct UserEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub orders: Vec<Order>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Order {
    pub order_id: Uuid,        // Unique identifier of order
    pub client_order_id: Uuid, // Unique identifier of order specified by client
    #[serde(with = "string_or_float")]
    pub cumulative_quantity: f64, // Amount the order is filled, in base currency
    #[serde(with = "string_or_float")]
    pub leaves_quantity: f64, // Amount remaining, in same currency as order was placed in (quote or base)
    #[serde(with = "string_or_float")]
    pub avg_price: f64, // Average filled price of the order so far
    #[serde(with = "string_or_float")]
    pub total_fees: f64, //Commission paid for the order
    pub status: OrderStatus,
    pub product_id: String, // The product ID for which this order was placed
    pub creation_time: DateTime<Utc>, // When the order was placed
    pub order_side: TradeSide, // Can be one of: BUY, SELL
    pub order_type: String, // Can be one of: Limit, Market, Stop Limit
}
