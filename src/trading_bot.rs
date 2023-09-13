use std::fmt;

use crate::{
    indicators::{bollinger_bands::BollingerBands, ema::Ema, macd::Macd, rsi::Rsi},
    model::Candlestick,
};

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

pub enum IndicatorType {
    Candlestick(Candlestick),
}

pub struct TradingBot {
    current_price: f64,
    current_candle_time: i64,
    time_count: usize,
    pub can_trade: bool,
    current_signal: TradeSignal,
    short_ema: Ema,
    long_ema: Ema,
    macd: Macd,
    rsi: Rsi,
    b_bands: BollingerBands,
}

impl TradingBot {
    pub fn new() -> Self {
        let short_ema = Ema::new(9);
        let long_ema = Ema::new(21);
        let macd = Macd::new(12, 26, 9);
        let rsi = Rsi::new(14);
        let b_bands = BollingerBands::new(20);

        TradingBot {
            current_price: 0.0,
            current_candle_time: 0,
            time_count: 0,
            can_trade: false,
            current_signal: TradeSignal::Hold,
            short_ema,
            long_ema,
            macd,
            rsi,
            b_bands,
        }
    }

    pub fn process_data(&mut self, data: IndicatorType) {
        match data {
            IndicatorType::Candlestick(candle_stick) => {
                if self.current_candle_time != candle_stick.start {
                    self.can_trade = true;
                    self.current_candle_time = candle_stick.start;
                }

                self.short_ema.update(candle_stick.close);
                self.long_ema.update(candle_stick.close);

                self.macd.update(candle_stick.close);
                self.rsi.update(candle_stick.close);
                self.b_bands.update(candle_stick.close);

                self.current_price = candle_stick.close;
            }
        }
    }

    pub fn check_trade_signal(&mut self) -> TradeSignal {
        if self.time_count < 6 {
            self.time_count += 1;
            return TradeSignal::Hold;
        }
        let short_ema = self.short_ema.prev_ema.unwrap_or(0.0);
        let long_ema = self.long_ema.prev_ema.unwrap_or(0.0);
        let macd_line = self.macd.get_macd().unwrap_or(0.0);
        let macd_signal = self.macd.get_signal().unwrap_or(0.0);
        let lower_band = self.b_bands.lower_band.unwrap_or(0.0);
        let upper_band = self.b_bands.upper_band.unwrap_or(0.0);
        let rsi = self.rsi.get_rsi().unwrap_or(0.0);

        if short_ema > long_ema
            && macd_line > macd_signal
            && self.current_price <= lower_band
            && rsi < 30.0
        {
            return TradeSignal::Buy;
        }

        if short_ema < long_ema
            && macd_line < macd_signal
            && self.current_price >= upper_band
            && rsi > 70.0
        {
            return TradeSignal::Sell;
        }

        TradeSignal::Hold
    }
}
