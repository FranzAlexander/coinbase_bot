use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{string_or_float, OrderStatus}; // I assume you're using the 'uuid' crate

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MarketMarketIoc {
    pub quote_size: Option<String>,
    pub base_size: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct LimitLimitGtd {
    pub base_size: Option<String>,
    pub limit_price: Option<String>,
    pub end_time: Option<String>,
    pub post_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OrderConfiguration {
    pub market_market_ioc: Option<MarketMarketIoc>, // Adjusted this line
    pub limit_limit_gtd: Option<LimitLimitGtd>,
}

#[derive(Deserialize, Debug)]
pub struct OrderResponse {
    pub success: bool,
    pub failure_reason: String,
    pub order_id: String,
    pub success_response: Option<SuccessResponse>,
    pub error_response: Option<ErrorResponse>,
    pub order_configuration: OrderConfiguration,
}

#[derive(Deserialize, Debug)]
pub struct SuccessResponse {
    pub order_id: String,
    pub product_id: String,
    pub side: String,
    pub client_order_id: String,
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub error_details: String,
    pub preview_failure_reason: String,
    pub new_order_failure_reason: Option<String>, // Adjusted this line
}

#[derive(Deserialize, Debug)]
pub struct CurrentOrderResponse {
    pub order: CurrentOrder,
}

#[derive(Deserialize, Debug)]
pub struct CurrentOrder {
    #[serde(with = "string_or_float")]
    pub average_filled_price: f64,
    pub cancel_message: Option<String>,
    pub client_order_id: String,
    pub completion_percentage: String,
    pub created_time: DateTime<Utc>,
    pub fee: Option<String>,
    #[serde(with = "string_or_float")]
    pub filled_size: f64,
    pub filled_value: String,
    pub is_liquidation: bool,
    pub number_of_fills: String,
    pub order_configuration: OrderConfiguration,
    pub order_id: String,
    pub order_placement_source: String,
    pub order_type: String,
    pub outstanding_hold_amount: Option<String>,
    pub pending_cancel: bool,
    pub product_id: String,
    pub product_type: String,
    pub reject_message: Option<String>,
    pub reject_reason: Option<String>, // Adjusted the name to match the provided data
    pub settled: bool,
    pub side: String,
    pub size_in_quote: bool,
    pub size_inclusive_of_fees: bool,
    pub status: String,
    pub time_in_force: String,
    pub total_fees: String,
    pub total_value_after_fees: String,
    pub trigger_status: Option<String>,
    pub user_id: String,
}
