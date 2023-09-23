use serde::{Deserialize, Serialize};
use uuid::Uuid; // I assume you're using the 'uuid' crate

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MarketMarketIoc {
    pub quote_size: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrderConfiguration {
    pub market_market_ioc: Option<MarketMarketIoc>, // Adjusted this line
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub error_details: String,
    pub preview_failure_reason: String,
    pub new_order_failure_reason: Option<String>, // Adjusted this line
}
