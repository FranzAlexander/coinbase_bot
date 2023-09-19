use serde::Serialize;

use super::TradeSide;

#[derive(Serialize)]
#[serde(rename = "snake_case")]
pub struct LimitLimitGtc {
    base_size: String,
    limit_price: String,
    post_only: bool,
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
