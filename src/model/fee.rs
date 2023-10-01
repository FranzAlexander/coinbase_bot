use serde::{Deserialize, Serialize};

use super::{string_or_float, string_or_float_opt};

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeData {
    #[serde(with = "string_or_float")]
    total_volume: f64,
    #[serde(with = "string_or_float")]
    total_fees: f64,
    pub fee_tier: FeeTier,
    margin_rate: Option<MarginRate>,
    goods_and_services_tax: Option<GoodsAndServicesTax>,
    #[serde(with = "string_or_float_opt")]
    advanced_trade_only_volume: Option<f64>,
    #[serde(with = "string_or_float_opt")]
    advanced_trade_only_fees: Option<f64>,
    #[serde(with = "string_or_float_opt")]
    coinbase_pro_volume: Option<f64>,
    #[serde(with = "string_or_float_opt")]
    coinbase_pro_fees: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeTier {
    pub pricing_tier: Option<String>,
    pub usd_from: Option<String>,
    pub usd_to: Option<String>,
    #[serde(with = "string_or_float")]
    pub taker_fee_rate: f64,
    #[serde(with = "string_or_float")]
    pub maker_fee_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MarginRate {
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoodsAndServicesTax {
    rate: Option<String>,
    #[serde(rename = "type")]
    tax_type: Option<TaxType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaxType {
    INCLUSIVE,
    EXCLUSIVE,
}
