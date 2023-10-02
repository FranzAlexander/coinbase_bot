use std::fmt;

use super::ema::Ema;

#[derive(Debug)]
pub struct Macd {
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
    macd_value: f64,
    signal: f64,
}

impl Macd {
    pub fn new(fast_length: usize, slow_length: usize, signal_length: usize) -> Self {
        Macd {
            fast_ema: Ema::new(fast_length),
            slow_ema: Ema::new(slow_length),
            signal_ema: Ema::new(signal_length),
            macd_value: 0.0,
            signal: 0.0,
        }
    }

    pub fn update(&mut self, price: f64) {
        if let (Some(fast_ema), Some(slow_ema)) =
            (self.fast_ema.update(price), self.slow_ema.update(price))
        {
            self.macd_value = fast_ema - slow_ema;
            self.signal = self.signal_ema.update(self.macd_value).unwrap_or(0.0);
        }
    }

    pub fn get_macd_line(&self) -> f64 {
        self.macd_value
    }

    pub fn get_signal_line(&self) -> f64 {
        self.signal
    }
}

impl fmt::Display for Macd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Value: {}, Signal: {}", self.macd_value, self.signal)
    }
}
