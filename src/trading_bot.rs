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

const MAX_MACD_SIGNAL_PERIOD: usize = 10;

pub struct TradingBot {
    pub price: f64,
    pub short_ema: Ema,
    pub long_ema: Ema,
    pub macd: Macd,
    pub obv: Obv,
    pub count: usize,
    pub macd_count: usize,
    pub current_macd_signal: TradeSignal,
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
            macd_count: 0,
            current_macd_signal: TradeSignal::Hold,
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

    pub fn get_signal(&mut self) -> TradeSignal {
        if self.count <= 35 {
            return TradeSignal::Hold;
        }

        let (macd_line, macd_signal) = match (self.macd.prev_ema, self.macd.get_signal()) {
            (Some(macd_line), Some(macd_signal)) => (macd_line, macd_signal),
            _ => return TradeSignal::Hold,
        };

        self.update_macd_signal_and_count(macd_line, macd_signal);

        // Return Hold if macd_count is 10 or more, irrespective of EMA conditions.
        if self.macd_count >= MAX_MACD_SIGNAL_PERIOD {
            return TradeSignal::Hold;
        }

        match (self.short_ema.prev_ema, self.long_ema.prev_ema) {
            (Some(short_ema), Some(long_ema)) => {
                if short_ema > long_ema && self.current_macd_signal == TradeSignal::Buy {
                    TradeSignal::Buy
                } else if short_ema < long_ema && self.current_macd_signal == TradeSignal::Sell {
                    TradeSignal::Sell
                } else {
                    TradeSignal::Hold
                }
            }
            _ => TradeSignal::Hold,
        }
    }

    fn update_macd_signal_and_count(&mut self, macd_line: f64, macd_signal: f64) {
        if macd_line > macd_signal && self.current_macd_signal == TradeSignal::Buy
            || macd_line < macd_signal && self.current_macd_signal == TradeSignal::Sell
        {
            self.macd_count += 1;
        } else {
            self.macd_count = 1;
            self.current_macd_signal = if macd_line > macd_signal {
                TradeSignal::Buy
            } else {
                TradeSignal::Sell
            };
        }
    }
}
