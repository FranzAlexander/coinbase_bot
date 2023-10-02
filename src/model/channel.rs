use crate::{
    coin::CoinSymbol,
    trading_bot::{IndicatorTimeframe, TradeSignal},
};

use super::event::MarketTradeEvent;

#[derive(Debug)]
pub struct IndicatorChannelMessage {
    pub timeframe: IndicatorTimeframe,
    pub symbol: CoinSymbol,
    pub trades: Vec<MarketTradeEvent>,
}

pub struct AccountChannelMessage {
    pub symbol: CoinSymbol,
    pub price: Option<f64>,
    pub signal: Option<TradeSignal>,
}
