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
    pub symbol: CoinSymbol,
    pub price: Option<f64>,
    pub signal: Option<TradeSignal>,
    pub atr: Option<f64>,
}
