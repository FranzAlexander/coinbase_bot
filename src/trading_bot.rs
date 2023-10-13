use std::collections::VecDeque;

use crate::{
    indicators::{
        atr::Atr,
        ema::Ema,
        macd::{self, Macd},
        rsi::Rsi,
    },
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

const ATR_MODIFIER: f64 = 1.5;

#[derive(Debug)]
#[allow(dead_code)]
pub struct TradingIndicator {
    macd: Macd,
    price_ema: Ema, // rsi: Rsi,
}

// Used to use macd and rsi.
// Now use macd and ema cross.

impl TradingIndicator {
    pub fn new() -> Self {
        let macd = Macd::new(12, 21, 9);
        // let rsi = Rsi::new(14);
        let price_ema = Ema::new(20);

        TradingIndicator { macd, price_ema }
    }

    pub fn update(&mut self, current_price: f64) {
        self.macd.update(current_price);
        self.price_ema.update(current_price);
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

    pub fn get_ema_signal(&self, price: f64) -> TradeSignal {
        let current_ema = self.price_ema.get_ema();

        if current_ema < price {
            TradeSignal::Buy
        } else if current_ema > price {
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
    }

    pub fn get_signal(&mut self, price: f64) -> TradeSignal {
        let ema_signal = self.long_trading.get_ema_signal(price);
        let macd_signal = self.long_trading.get_macd_signal();

        if ema_signal == TradeSignal::Buy && macd_signal == TradeSignal::Buy {
            TradeSignal::Buy
        } else if ema_signal == TradeSignal::Sell && macd_signal == TradeSignal::Sell {
            self.can_trade = true;
            TradeSignal::Sell
        } else {
            TradeSignal::Hold
        }
    }

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
