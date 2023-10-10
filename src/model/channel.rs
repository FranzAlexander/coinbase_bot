use smallvec::SmallVec;

use crate::{coin::CoinSymbol, trading_bot::TradeSignal};

use super::event::CandleEvent;

#[derive(Debug)]
pub struct IndicatorChannelMessage {
    pub symbol: CoinSymbol,
    pub candles: SmallVec<[CandleEvent; 1]>,
}

#[derive(Debug)]
pub struct AccountChannelMessage {
    pub symbol: CoinSymbol,
    pub signal: TradeSignal,
    pub atr: Option<f64>,
    pub high: f64,
}
