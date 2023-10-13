use std::collections::VecDeque;

use crate::{
    indicators::{atr::Atr, ema::Ema, macd::Macd, rsi::Rsi},
    model::event::Candlestick,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

const RSI_OVERSOLD: f64 = 40.0;
const RSI_OVERBROUGHT: f64 = 60.0;

const MAX_CROSS_PERIOD: usize = 3;

const ATR_MODIFIER: f64 = 1.0;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TradingIndicator {
    macd: Macd,
    // rsi: Rsi,
    short_ema: Ema,
    long_ema: Ema,
}

// Used to use macd and rsi.
// Now use macd and ema cross.

impl TradingIndicator {
    pub fn new() -> Self {
        let macd = Macd::new(12, 21, 9);
        // let rsi = Rsi::new(14);
        let short_ema = Ema::new(9);
        let long_ema = Ema::new(12);

        TradingIndicator {
            macd,
            short_ema,
            long_ema,
        }
    }

    pub fn update(&mut self, current_price: f64) {
        self.macd.update(current_price);
        self.short_ema.update(current_price);
        self.long_ema.update(current_price);
    }

    // pub fn get_rsi_signal(&self) -> TradeSignal {
    //     if let Some(rsi) = self.rsi.get_current_rsi() {
    //         if rsi >= RSI_OVERBROUGHT {
    //             TradeSignal::Sell
    //         } else if rsi <= RSI_OVERSOLD {
    //             TradeSignal::Buy
    //         } else {
    //             TradeSignal::Hold
    //         }
    //     } else {
    //         TradeSignal::Hold
    //     }
    // }

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

    pub fn get_ema_cross(&self) -> TradeSignal {
        let short_ema = self.short_ema.get_ema();
        let long_ema = self.long_ema.get_ema();

        if short_ema > long_ema {
            TradeSignal::Buy
        } else if short_ema < long_ema {
            TradeSignal::Sell
        } else {
            TradeSignal::Hold
        }
    }
}

#[derive(Debug)]
pub struct TradingBot {
    long_trading: TradingIndicator,
    atr: Atr,
    // latest_rsi_signals: VecDeque<TradeSignal>,
    can_trade: bool,
    pub candle: Candlestick,
    pub initialise: bool,
}

impl TradingBot {
    pub fn new() -> Self {
        let long_trading = TradingIndicator::new();
        let atr = Atr::new(14);

        TradingBot {
            long_trading,
            atr,
            // latest_rsi_signals: VecDeque::with_capacity(3),
            can_trade: true,
            candle: Candlestick {
                start: 0,
                low: 0.0,
                high: 0.0,
                open: 0.0,
                close: 0.0,
                volume: 0.0,
            },
            initialise: false,
        }
    }

    pub fn one_minute_update(&mut self, candle: Candlestick) {
        self.long_trading.update(candle.close);
        self.atr.update(candle.high, candle.low, candle.close);

        // if self.latest_rsi_signals.len() > MAX_CROSS_PERIOD {
        //     self.latest_rsi_signals.pop_back();
        // }

        // self.latest_rsi_signals
        //     .push_front(self.long_trading.get_rsi_signal());
    }

    pub fn get_signal(&mut self) -> TradeSignal {
        // let rsi_signal = self.check_rsi_signal();
        // let macd_signal = self.long_trading.get_macd_signal();
        // if rsi_signal == TradeSignal::Buy && macd_signal == TradeSignal::Buy {
        //     TradeSignal::Buy
        // } else if rsi_signal == TradeSignal::Sell && macd_signal == TradeSignal::Sell {
        //     self.can_trade = true;
        //     TradeSignal::Sell
        // } else {
        //     TradeSignal::Hold
        // }

        let ema_cross = self.long_trading.get_ema_cross();
        let macd_signal = self.long_trading.get_macd_signal();

        if ema_cross == TradeSignal::Buy && macd_signal == TradeSignal::Buy {
            TradeSignal::Buy
        } else if ema_cross == TradeSignal::Sell && macd_signal == TradeSignal::Sell {
            self.can_trade = true;
            TradeSignal::Sell
        } else {
            TradeSignal::Hold
        }
    }

    // pub fn check_rsi_signal(&mut self) -> TradeSignal {
    //     let buy_signal = self.latest_rsi_signals.contains(&TradeSignal::Buy);
    //     if buy_signal {
    //         TradeSignal::Buy
    //     } else {
    //         *self.latest_rsi_signals.back().unwrap()
    //     }
    // }

    pub fn get_atr_value(&self) -> Option<f64> {
        self.atr.get_atr().map(|atr| atr * ATR_MODIFIER)
    }

    pub fn get_can_trade(&self) -> bool {
        self.can_trade
    }

    pub fn set_can_trade(&mut self, can_trade: bool) {
        self.can_trade = can_trade
    }
}

pub struct IndicatorResult {
    pub signal: TradeSignal,
    pub atr: Option<f64>,
    pub high: f64,
}
