use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{string_or_float, TradeSide};

#[derive(Serialize, Deserialize)]
#[serde(rename = "snake_case")]
pub struct MarketMarketIoc {
    pub quote_size: Option<String>,
}

#[derive(Deserialize)]
pub struct OrderResponse {
    pub success: bool,
    pub failure_reason: String,
    pub order_id: Uuid,
    pub success_response: SuccessResponse,
    pub error_response: ErrorResponse,
    pub market_market_ioc: MarketMarketIoc,
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
