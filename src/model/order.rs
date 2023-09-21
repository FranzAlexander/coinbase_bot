use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{string_or_float, TradeSide};

#[derive(Serialize, Deserialize)]
#[serde(rename = "snake_case")]
pub struct LimitLimitGtc {
    #[serde(with = "string_or_float")]
    pub base_size: f64,
    #[serde(with = "string_or_float")]
    pub limit_price: f64,
    pub post_only: bool,
}

#[derive(Serialize)]
#[serde(rename = "snake_case")]
pub struct OrderConfiguration {
    market_market_ioc: LimitLimitGtc,
}

#[derive(Serialize)]
pub struct CreateOrder {
    client_order_id: String,
    product_id: String,
    side: TradeSide, // You can change this to an enum as well if you always match the order side with MarketOrder
    order_configuration: OrderConfiguration,
}

#[derive(Deserialize)]
pub struct OrderResponse {
    pub success: bool,
    pub failure_reason: Option<String>,
    pub order_id: Uuid,
    pub success_response: SuccessResponse,
    pub error_response: ErrorResponse,
    pub limit_limit_gtc: LimitLimitGtc,
}

#[derive(Deserialize)]
pub struct SuccessResponse {
    pub order_id: Uuid,
    pub product_id: String,
    pub side: String,
    pub client_order_id: Uuid,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub error_details: String,
    pub preview_failure_reason: String,
    pub new_order_failure_reason: String,
}
