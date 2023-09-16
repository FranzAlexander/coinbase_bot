use core::fmt;

use crate::model::{Candlestick, CandlestickEvent};

use self::{bollinger_bands::BollingerBands, ema::Ema, macd::Macd, rsi::Rsi};

// mod adx;
pub mod bollinger_bands;
pub mod ema;
pub mod macd;
pub mod order_book;
pub mod rsi;

pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

impl fmt::Display for TradeSignal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TradeSignal::Buy => write!(f, "Buy"),
            TradeSignal::Sell => write!(f, "Sell"),
            TradeSignal::Hold => write!(f, "Hold"),
        }
    }
}
