use std::{sync::Arc, thread};

use tokio::sync::{mpsc, Mutex};

use crate::{
    indicators::{ema::Ema, macd::Macd, obv::Obv},
    model::candlestick::Candlestick,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

pub struct TradingBot {
    pub price: f64,
    pub short_ema: Ema,
    pub long_ema: Ema,
    pub macd: Macd,
    pub macd_current_signal: TradeSignal,
    pub obv: Obv,
}

impl TradingBot {
    pub fn new() -> Self {
        let short_ema = Ema::new(9);
        let long_ema = Ema::new(12);
        let macd = Macd::new(12, 26, 9);
        let obv = Obv::new();

        TradingBot {
            price: 0.0,
            short_ema,
            long_ema,
            macd,
            macd_current_signal: TradeSignal::Hold,
            obv,
        }
    }

    pub fn update_bot(&mut self, candle: Candlestick) {
        self.short_ema.update(candle.close);
        self.long_ema.update(candle.close);
        self.macd.update(candle.close);
        self.obv.update(candle.close, candle.volume);
        self.price = candle.close;
    }

    pub fn get_signal(&self) -> TradeSignal {
        let short_ema = self.short_ema.prev_ema.unwrap_or(0.0);
        let long_ema = self.long_ema.prev_ema.unwrap_or(0.0);

        if short_ema > long_ema {
            return TradeSignal::Buy;
        }

        if short_ema < long_ema {
            return TradeSignal::Sell;
        }

        TradeSignal::Hold
    }
}
