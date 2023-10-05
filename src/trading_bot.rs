use std::collections::VecDeque;

use crate::{
    candlestick::Candlestick,
    indicators::{atr::Atr, macd::Macd, rsi::Rsi},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
}

const RSI_OVERSOLD: f64 = 40.0;
const RSI_OVERBROUGHT: f64 = 60.0;
// const RSI_CROSS_BUY_CHECK: f64 = 45.0;

const MAX_CROSS_PERIOD: usize = 3;
const MIN_CANDLE_PROCCESSED: usize = 2;

const ATR_MODIFIER: f64 = 1.25;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
        let macd = Macd::new(12, 21, 9);
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

    pub fn get_rsi(&self) -> Option<f64> {
        self.rsi.get_current_rsi()
    }
}

#[derive(Debug)]
pub struct TradingBot {
    long_trading: TradingIndicator,
    atr: Atr,
    count: usize,
    latest_rsi_signals: VecDeque<TradeSignal>,
}

impl TradingBot {
    pub fn new() -> Self {
        let long_trading = TradingIndicator::new(IndicatorTimeframe::OneMinute);
        let atr = Atr::new(14);

        TradingBot {
            long_trading,
            atr,
            count: 0,
            latest_rsi_signals: VecDeque::with_capacity(3),
        }
    }

    pub fn one_minute_update(&mut self, candle: Candlestick) {
        self.long_trading.update(candle.close);
        self.atr.update(candle.high, candle.low, candle.close);

        if self.latest_rsi_signals.len() > MAX_CROSS_PERIOD {
            self.latest_rsi_signals.pop_back();
        }

        self.latest_rsi_signals
            .push_front(self.long_trading.get_rsi_signal());

        if self.count <= MIN_CANDLE_PROCCESSED {
            self.count += 1;
        }
    }

    pub fn get_signal(&mut self, timeframe: IndicatorTimeframe) -> TradeSignal {
        if timeframe == IndicatorTimeframe::OneMinute {
            if self.count > MIN_CANDLE_PROCCESSED {
                let rsi_signal = self.check_rsi_signal();
                let macd_signal = self.long_trading.get_macd_signal();
                if rsi_signal == TradeSignal::Buy && macd_signal == TradeSignal::Buy {
                    return TradeSignal::Buy;
                } else if rsi_signal == TradeSignal::Sell && macd_signal == TradeSignal::Sell {
                    return TradeSignal::Sell;
                } else {
                    return TradeSignal::Hold;
                }
            }
            TradeSignal::Hold
        } else {
            TradeSignal::Hold
        }
    }

    pub fn get_macd_signal(&self) -> TradeSignal {
        self.long_trading.get_macd_signal()
    }

    pub fn check_rsi_signal(&mut self) -> TradeSignal {
        let buy_signal = self.latest_rsi_signals.contains(&TradeSignal::Buy);
        if buy_signal {
            TradeSignal::Buy
        } else {
            *self.latest_rsi_signals.back().unwrap()
        }
        // let current_signal = self.long_trading.get_rsi_signal();

        // if current_signal == self.last_rsi_cross {
        //     self.since_last_cross += 1;
        //     if self.since_last_cross > MAX_CROSS_PERIOD - 1 {
        //         if let Some(rsi) = self.long_trading.get_rsi() {
        //             if rsi <= RSI_CROSS_BUY_CHECK {
        //                 self.last_rsi_cross = current_signal;
        //             }
        //         }
        //     } else if self.since_last_cross > MAX_CROSS_PERIOD {
        //         self.last_rsi_cross = TradeSignal::Hold;
        //     } else {
        //         self.last_rsi_cross = current_signal;
        //     }
        // } else {
        //     self.since_last_cross = 0;
        //     self.last_rsi_cross = current_signal;
        // }

        // self.last_rsi_cross
    }

    pub fn get_rsi_signal(&self) -> TradeSignal {
        self.long_trading.get_rsi_signal()
    }

    pub fn get_atr_value(&self) -> Option<f64> {
        if let Some(atr) = self.atr.get_atr() {
            Some(atr * ATR_MODIFIER)
        } else {
            None
        }
    }
}
