use std::fmt;

use crate::{
    indicators::{
        ema::Ema,
        macd::Macd,
        obv::Obv,
        stoch_rsi::{self, StochRsi},
    },
    model::candlestick::Candlestick,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

const MAX_MACD_SIGNAL_PERIOD: usize = 10;
const MIN_CANDLE_PROCCESSED: usize = 20;

pub struct TradingBot {
    pub price: f64,
    macd: Macd,     // Uses kline close
    low_macd: Macd, // Uses kline low
    pub obv: Obv,
    pub stoch_rsi: StochRsi,
    pub count: usize,
    pub current_macd_signal: TradeSignal,
}

impl TradingBot {
    pub fn new() -> Self {
        let macd = Macd::new(9, 12, 7);
        let low_macd = Macd::new(9, 12, 7);
        let obv = Obv::new();
        let stoch_rsi = StochRsi::new(9, 12, 3, 3);

        TradingBot {
            price: 0.0,
            macd,
            low_macd,
            obv,
            stoch_rsi,
            count: 0,
            current_macd_signal: TradeSignal::Hold,
        }
    }

    pub fn update_bot(&mut self, candle: Candlestick) {
        self.macd.update(candle.close);
        self.low_macd.update(candle.low.unwrap_or(0.00001));
        self.obv.update(candle.close, candle.volume);
        self.stoch_rsi.update(candle.close, self.price);
        self.price = candle.close;

        if self.count <= MIN_CANDLE_PROCCESSED {
            self.count += 1;
        }
    }

    pub fn get_signal(&mut self) -> TradeSignal {
        if self.count <= MIN_CANDLE_PROCCESSED {
            return TradeSignal::Hold;
        }

        let (low_macd_line, low_macd_signal) =
            match (self.low_macd.prev_ema, self.low_macd.get_signal()) {
                (Some(macd_line), Some(macd_signal)) => (macd_line, macd_signal),
                _ => return TradeSignal::Hold,
            };

        let low_trade_signal = self.get_macd_signal(low_macd_line, low_macd_signal);

        let (macd_line, macd_signal) = match (self.macd.prev_ema, self.macd.get_signal()) {
            (Some(macd_line), Some(macd_signal)) => (macd_line, macd_signal),
            _ => return TradeSignal::Hold,
        };

        let macd_trade_signal = self.get_macd_signal(macd_line, macd_signal);

        let (avg_k, avg_d) = match (self.stoch_rsi.get_avg_k(), self.stoch_rsi.get_avg_d()) {
            (Some(avg_k), Some(avg_d)) => (avg_k, avg_d),
            _ => return TradeSignal::Hold,
        };

        let obv_trend = self.obv.get_trend();

        if macd_trade_signal == TradeSignal::Buy
            && low_trade_signal == TradeSignal::Buy
            && avg_k > avg_d
        {
            TradeSignal::Buy
        } else if macd_trade_signal == TradeSignal::Sell && avg_k < avg_d {
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
        write!(f, "Macd Line: {}, Macd Signal: {}, Low Macd Line: {}, Low Macd Signal: {}, avg k: {}, avg d: {}",
        self.macd.prev_ema.unwrap_or(0.0),
        self.macd.get_signal().unwrap_or(0.0),
        self.low_macd.prev_ema.unwrap_or(0.0),
        self.low_macd.get_signal().unwrap_or(0.0),
        self.stoch_rsi.get_avg_k().unwrap_or(0.0),
        self.stoch_rsi.get_avg_d().unwrap_or(0.0))
    }
}
