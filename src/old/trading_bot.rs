use std::fmt;

use crate::{
    indicators::{
        bollinger_bands::BollingerBands, ema::Ema, macd::Macd, order_book::OrderBook, rsi::Rsi,
    },
    model::{Candlestick, L2Data, Level2Event},
};

#[derive(Debug)]
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
    L2Data(Level2Event),
}

#[derive(Debug)]
pub struct TradingBot {
    pub current_price: f64,
    current_candle_time: i64,
    time_count: usize,
    pub can_trade: bool,
    current_signal: TradeSignal,
    short_ema: Ema,
    long_ema: Ema,
    macd: Macd,
    rsi: Rsi,
    b_bands: BollingerBands,
    pub order_book: OrderBook,
}

impl TradingBot {
    pub fn new() -> Self {
        let short_ema = Ema::new(9);
        let long_ema = Ema::new(21);
        let macd = Macd::new(12, 26, 9);
        let rsi = Rsi::new(14);
        let b_bands = BollingerBands::new(20);
        let order_book = OrderBook::new();

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
            order_book,
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
            IndicatorType::L2Data(l2_data) => {
                for update in l2_data.updates {
                    self.order_book.process_side_update(update)
                }
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

        let (strong_support, strong_resistance) =
            self.order_book.identify_support_and_resistance(0.5);

        if short_ema > long_ema
            && macd_line > macd_signal
            && (self.current_price <= lower_band || self.current_price <= strong_support)
            && rsi < 40.0
        {
            return TradeSignal::Buy;
        }

        if short_ema < long_ema
            && macd_line < macd_signal
            && (self.current_price >= upper_band || self.current_price >= strong_resistance)
            && rsi > 60.0
        {
            return TradeSignal::Sell;
        }

        TradeSignal::Hold
    }
}
