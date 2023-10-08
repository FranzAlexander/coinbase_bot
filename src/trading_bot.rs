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

// Look into price action.

const RSI_OVERSOLD: f64 = 40.0;
const RSI_OVERBROUGHT: f64 = 60.0;
// const RSI_CROSS_BUY_CHECK: f64 = 45.0;

const MAX_CROSS_PERIOD: usize = 3;
const MIN_CANDLE_PROCCESSED: usize = 2;

const ATR_MODIFIER: f64 = 1.0;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IndicatorTimeframe {
    OneMinute,
    FiveMinute,
}

#[derive(Debug)]
#[allow(dead_code)]
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

    pub fn check_rsi_signal(&mut self) -> TradeSignal {
        let buy_signal = self.latest_rsi_signals.contains(&TradeSignal::Buy);
        if buy_signal {
            TradeSignal::Buy
        } else {
            *self.latest_rsi_signals.back().unwrap()
        }
    }

    pub fn get_atr_value(&self) -> Option<f64> {
        self.atr.get_atr().map(|atr| atr * ATR_MODIFIER)
    }
}

pub struct IndicatorGroup {
    pub timeframe: IndicatorTimeframe,
    pub trading_bot: TradingBot,
    pub candle: Candlestick,
    pub initialise: bool,
}
