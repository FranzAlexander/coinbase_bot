use crate::{coin::CoinSymbol, trading_bot::TradeSignal};

pub struct AccountChannelMessage {
    pub symbol: CoinSymbol,
    pub price: Option<f64>,
    pub signal: Option<TradeSignal>,
}
