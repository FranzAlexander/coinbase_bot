use crate::model::OneMinuteCandle;

use self::{adx::Adx, bollinger_bands::BollingerBands, ema::Ema, macd::Macd, rsi::Rsi};

mod adx;
mod bollinger_bands;
pub mod ema;
pub mod macd;
pub mod rsi;

pub enum Signal {
    Buy,
    Sell,
    Hold,
}

pub enum IndicatorType {
    Candlestick(OneMinuteCandle),
}

#[derive(Debug)]
pub struct BotIndicator {
    short_ema: Ema,
    long_ema: Ema,
    macd_short_ema: Ema,
    macd: Macd,
    pub rsi: Rsi,
    pub adx: Adx,
    pub b_bands: BollingerBands,
}

impl BotIndicator {
    pub fn new() -> Self {
        let short_ema = Ema::new(9);
        let macd_short_ema = Ema::new(12);
        let long_ema = Ema::new(26);
        let macd = Macd::new(9);
        let rsi = Rsi::new(14);
        let adx = Adx::new(14, 14);
        let b_bands = BollingerBands::new(20);

        BotIndicator {
            short_ema,
            long_ema,
            macd_short_ema,
            macd,
            rsi,
            adx,
            b_bands,
        }
    }

    pub fn process_data(&mut self, data: IndicatorType) {
        match data {
            IndicatorType::Candlestick(candle_stick) => {
                self.short_ema.update(candle_stick.close);
                self.macd_short_ema.update(candle_stick.close);
                self.long_ema.update(candle_stick.close);

                if let (Some(short_ema), Some(long_ema)) =
                    (self.macd_short_ema.get_ema(), self.long_ema.get_ema())
                {
                    self.macd.update(short_ema, long_ema);
                }

                self.rsi.update(candle_stick.close);

                self.adx
                    .update(candle_stick.high, candle_stick.low, candle_stick.close);

                self.b_bands.update(candle_stick.close);
            }
        }
    }

    pub fn get_short_ema(&self) -> Option<f64> {
        self.short_ema.get_ema()
    }

    pub fn get_short_macd_ema(&self) -> Option<f64> {
        self.macd_short_ema.get_ema()
    }

    pub fn get_long_ema(&self) -> Option<f64> {
        self.long_ema.get_ema()
    }

    pub fn get_macd_histogram(&self) -> Option<f64> {
        self.macd.get_histogram()
    }

    // pub fn check(&self) -> Signal {}
}
