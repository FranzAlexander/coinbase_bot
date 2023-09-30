use std::fmt;

use crate::{
    indicators::{ema::Ema, macd::Macd, obv::Obv, rsi::Rsi},
    model::candlestick::Candlestick,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

const MACD_FIVE_TIMEFRAME: (usize, usize, usize) = (12, 26, 9);

const MAX_MACD_SIGNAL_PERIOD: usize = 10;
const MIN_CANDLE_PROCCESSED: usize = 20;

pub struct TradingBot {
    price: f64,
    macd: Macd,
    rsi: Rsi,
    count: usize,
    current_macd_signal: TradeSignal,
}

impl TradingBot {
    pub fn new() -> Self {
        let macd = Macd::new(9, 12, 7);
        let rsi = Rsi::new(14);

        TradingBot {
            price: 0.0,
            macd,
            rsi,
            count: 0,
            current_macd_signal: TradeSignal::Hold,
        }
    }

    pub fn update_bot(&mut self, candle: Candlestick) {
        self.macd.update(candle.close);
        self.rsi.update(candle.close);

        self.price = candle.close;

        if self.count <= MIN_CANDLE_PROCCESSED {
            self.count += 1;
        }
    }

    pub fn get_signal(&mut self) -> TradeSignal {
        if self.count <= MIN_CANDLE_PROCCESSED {
            return TradeSignal::Hold;
        }

        let (macd_line, macd_signal) = (self.macd.get_macd_line(), self.macd.get_signal_line());

        let macd_trade_signal = self.get_macd_signal(macd_line, macd_signal);

        if macd_trade_signal == TradeSignal::Buy {
            TradeSignal::Buy
        } else if macd_trade_signal == TradeSignal::Sell {
            TradeSignal::Sell
        } else {
            TradeSignal::Hold
        }
    }

    fn get_macd_signal(&self, macd_line: f64, macd_signal: f64) -> TradeSignal {
        if macd_line > macd_signal {
            TradeSignal::Buy
        } else {
            TradeSignal::Sell
        }
    }
}

impl fmt::Display for TradingBot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Macd Line: {}, Macd Signal: {}",
            self.macd.get_macd_line(),
            self.macd.get_signal_line(),
        )
    }
}
