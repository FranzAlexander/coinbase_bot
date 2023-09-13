use core::fmt;

use crate::model::{Candlestick, CandlestickEvent, OneMinuteCandle};

use self::{bollinger_bands::BollingerBands, ema::Ema, macd::Macd, rsi::Rsi};

// mod adx;
pub mod bollinger_bands;
pub mod ema;
pub mod macd;
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

pub enum IndicatorType {
    Candlestick(Candlestick),
}

// #[derive(Debug)]
// pub struct BotIndicator {
//     price: f64,
//     short_ema: Ema,
//     long_ema: Ema,
//     macd_short_ema: Ema,
//     macd_long_ema: Ema,
//     macd: Macd,
//     pub rsi: Rsi,
//     pub b_bands: BollingerBands,
// }

// impl BotIndicator {
//     pub fn new() -> Self {
//         let short_ema = Ema::new(9);
//         let long_ema = Ema::new(21);
//         let macd_short_ema = Ema::new(12);
//         let macd_long_ema = Ema::new(26);
//         let macd = Macd::new(9);
//         let rsi = Rsi::new(14);
//         let b_bands = BollingerBands::new(20);

//         BotIndicator {
//             price: 0.0,
//             short_ema,
//             long_ema,
//             macd_short_ema,
//             macd_long_ema,
//             macd,
//             rsi,
//             b_bands,
//         }
//     }

//     pub fn process_data(&mut self, data: IndicatorType) {
//         match data {
//             IndicatorType::Candlestick(candle_stick) => {
//                 self.price = candle_stick.close;
//                 self.short_ema.update(candle_stick.close);
//                 self.long_ema.update(candle_stick.close);

//                 self.macd_short_ema.update(candle_stick.close);
//                 self.macd_long_ema.update(candle_stick.close);

//                 if let (Some(short_ema), Some(long_ema)) =
//                     (self.macd_short_ema.get_ema(), self.macd_long_ema.get_ema())
//                 {
//                     self.macd.update(short_ema, long_ema);
//                 }

//                 self.rsi.update(candle_stick.close);

//                 self.b_bands.update(candle_stick.close);
//             }
//         }
//     }

//     pub fn get_short_ema(&self) -> Option<f64> {
//         self.short_ema.get_ema()
//     }

//     pub fn get_short_macd_ema(&self) -> Option<f64> {
//         self.macd_short_ema.get_ema()
//     }

//     pub fn get_long_ema(&self) -> Option<f64> {
//         self.long_ema.get_ema()
//     }

//     pub fn get_macd_histogram(&self) -> Option<f64> {
//         self.macd.get_histogram()
//     }

//     pub fn check_trade_signal(&self) -> TradeSignal {
//         let short_ema = self.short_ema.prev_ema.unwrap_or(0.0);
//         let long_ema = self.long_ema.prev_ema.unwrap_or(0.0);
//         let macd_line = self.macd.get_macd().unwrap_or(0.0);
//         let macd_signal = self.macd.get_signal().unwrap_or(0.0);
//         let lower_band = self.b_bands.lower_band.unwrap_or(0.0);
//         let upper_band = self.b_bands.upper_band.unwrap_or(0.0);

//         if short_ema > long_ema && macd_line > macd_signal && self.price <= lower_band {
//             return TradeSignal::Buy;
//         }

//         if short_ema < long_ema && macd_line < macd_signal && self.price >= upper_band {
//             return TradeSignal::Sell;
//         }

//         TradeSignal::Hold
//     }

//     // pub fn check(&self) -> Signal {}
// }
