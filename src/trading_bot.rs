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
    pub obv: Obv,
    pub count: usize,
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
            obv,
            count: 0,
        }
    }

    pub fn update_bot(&mut self, candle: Candlestick) {
        self.short_ema.update(candle.close);
        self.long_ema.update(candle.close);
        self.macd.update(candle.close);
        self.obv.update(candle.close, candle.volume);
        self.price = candle.close;

        if self.count <= 35 {
            self.count += 1;
        }
    }

    pub fn get_signal(&self) -> TradeSignal {
        if self.count <= 35 {
            return TradeSignal::Hold;
        }

        if let (Some(short_ema), Some(long_ema), Some(macd_line), Some(macd_signal)) = (
            self.short_ema.prev_ema,
            self.long_ema.prev_ema,
            self.macd.prev_ema,
            self.macd.get_signal(),
        ) {
            if short_ema > long_ema && macd_line > macd_signal {
                return TradeSignal::Buy;
            }

            if short_ema < long_ema && macd_line < macd_signal {
                return TradeSignal::Sell;
            }

            TradeSignal::Hold
        } else {
            TradeSignal::Hold
        }
    }
}
