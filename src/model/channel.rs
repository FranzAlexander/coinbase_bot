use smallvec::SmallVec;

use crate::{
    coin::CoinSymbol,
    trading_bot::{IndicatorTimeframe, TradeSignal},
};

use super::event::MarketTradeEvent;

#[derive(Debug)]
pub struct IndicatorChannelMessage {
    pub symbol: CoinSymbol,
    pub trades: SmallVec<[MarketTradeEvent; 1]>,
}

#[derive(Debug)]
pub struct AccountChannelMessage {
    pub timeframe: IndicatorTimeframe,
    pub symbol: CoinSymbol,
    pub start: i64,
    pub end: i64,
    pub signal: TradeSignal,
    pub atr: Option<f64>,
    pub high: f64,
}
