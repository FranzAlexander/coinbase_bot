use crate::{
    coin::CoinSymbol,
    trading_bot::{IndicatorTimeframe, TradeSignal},
};

use super::event::MarketTradeEvent;

#[derive(Debug)]
pub struct IndicatorChannelMessage {
    pub symbol: CoinSymbol,
    pub trades: Vec<MarketTradeEvent>,
}

#[derive(Debug)]
pub struct AccountChannelMessage {
    pub timeframe: IndicatorTimeframe,
    pub symbol: CoinSymbol,
    pub start: i64,
    pub end: i64,
    // pub price: Option<f64>,
    pub signal: TradeSignal,
    pub atr: Option<f64>,
    pub high: f64,
}
