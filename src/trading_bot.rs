use std::fmt;

use crate::{
    indicators::{atr::Atr, ema::Ema, macd::Macd, rsi::Rsi},
    model::{candlestick::Candlestick, event::MarketTrade},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

const RSI_OVERSOLD: f64 = 40.0;
const RSI_OVERBROUGHT: f64 = 60.0;

const MAX_MACD_SIGNAL_PERIOD: usize = 4;
const MIN_CANDLE_PROCCESSED: usize = 20;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IndicatorTimeframe {
    PerTrade,
    OneMinute,
}

#[derive(Debug)]
pub struct TradingIndicator {
    timeframe: IndicatorTimeframe,
    macd: Macd,
    rsi: Rsi,
}

impl TradingIndicator {
    pub fn new(timeframe: IndicatorTimeframe) -> Self {
        let macd = Macd::new(9, 12, 7);
        let rsi = Rsi::new(14);

        TradingIndicator {
            timeframe,
            macd,
            rsi,
        }
    }

    pub fn update(&mut self, current_price: f64) {
        self.macd.update(current_price);
        self.rsi.update(current_price);
    }

    pub fn get_rsi_signal(&self) -> TradeSignal {
        if let Some(rsi) = self.rsi.get_current_rsi() {
            if rsi >= RSI_OVERBROUGHT {
                TradeSignal::Sell
            } else if rsi <= RSI_OVERSOLD {
                TradeSignal::Buy
            } else {
                TradeSignal::Hold
            }
        } else {
            TradeSignal::Hold
        }
    }

    pub fn get_macd_signal(&self) -> TradeSignal {
        let macd_line = self.macd.get_macd_line();
        let macd_signal = self.macd.get_signal_line();

        if macd_line > macd_signal {
            TradeSignal::Buy
        } else if macd_line < macd_signal {
            TradeSignal::Sell
        } else {
            TradeSignal::Hold
        }
    }
}
#[derive(Debug)]
pub struct TradingBot {
    pub short_trading: TradingIndicator,
    long_trading: TradingIndicator,
    atr: Atr,
    count: usize,
    last_rsi_cross: TradeSignal, // price: f64,
                                 // macd: Macd,
                                 // rsi: Rsi,
                                 // count: usize,
                                 // current_macd_signal: TradeSignal,
}

impl TradingBot {
    pub fn new() -> Self {
        let short_trading = TradingIndicator::new(IndicatorTimeframe::PerTrade);
        let long_trading = TradingIndicator::new(IndicatorTimeframe::OneMinute);
        let atr = Atr::new(14);

        TradingBot {
            short_trading,
            long_trading,
            atr,
            count: 0,
            last_rsi_cross: TradeSignal::Hold,
        }
    }

    pub fn per_trade_update(&mut self, trade: MarketTrade) {
        self.short_trading.update(trade.price);
    }

    pub fn one_minute_update(&mut self, candle: Candlestick) {
        self.long_trading.update(candle.close);
        self.atr
            .update(candle.high, candle.low.unwrap(), candle.close);

        if self.count <= MIN_CANDLE_PROCCESSED {
            self.count += 1;
        }
    }

    pub fn get_signal(&mut self, timeframe: IndicatorTimeframe) -> TradeSignal {
        if timeframe == IndicatorTimeframe::PerTrade {
            self.short_trading.get_rsi_signal()
        } else {
            if self.count <= MIN_CANDLE_PROCCESSED {
                return TradeSignal::Hold;
            }
            TradeSignal::Hold
        }
    }

    pub fn check_rsi_signal(&mut self) {}

    // pub fn get_signal(&mut self) -> TradeSignal {
    //     if self.count <= MIN_CANDLE_PROCCESSED {
    //         return TradeSignal::Hold;
    //     }

    //     let (macd_line, macd_signal) = (self.macd.get_macd_line(), self.macd.get_signal_line());

    //     let macd_trade_signal = self.get_macd_signal(macd_line, macd_signal);

    //     if macd_trade_signal == TradeSignal::Buy {
    //         TradeSignal::Buy
    //     } else if macd_trade_signal == TradeSignal::Sell {
    //         TradeSignal::Sell
    //     } else {
    //         TradeSignal::Hold
    //     }
    // }

    // fn get_macd_signal(&self, macd_line: f64, macd_signal: f64) -> TradeSignal {
    //     if macd_line > macd_signal {
    //         TradeSignal::Buy
    //     } else {
    //         TradeSignal::Sell
    //     }
    // }
}

// impl fmt::Display for TradingBot {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "Macd Line: {}, Macd Signal: {}",
//             self.macd.get_macd_line(),
//             self.macd.get_signal_line(),
//         )
//     }
// }
