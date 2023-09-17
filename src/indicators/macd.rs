use super::ema::Ema;

#[derive(Debug)]
pub struct Macd {
    short_ema: Ema,
    long_ema: Ema,
    signal: Ema,
    prev_ema: Option<f64>,
}

impl Macd {
    pub fn new(short_period: usize, long_period: usize, signal_period: usize) -> Self {
        Macd {
            short_ema: Ema::new(short_period),
            long_ema: Ema::new(long_period),
            signal: Ema::new(signal_period),
            prev_ema: None,
        }
    }

    pub fn update(&mut self, price: f64) {
        self.short_ema.update(price);
        self.long_ema.update(price);

        if let (Some(fast_ema), Some(slow_ema)) =
            (self.short_ema.get_ema(), self.long_ema.get_ema())
        {
            let macd_value = fast_ema - slow_ema;
            self.prev_ema = Some(macd_value);
            self.signal.update(macd_value);
        }
    }

    pub fn get_signal(&self) -> Option<f64> {
        self.signal.get_ema()
    }

    pub fn get_histogram(&self) -> Option<f64> {
        if let Some(macd_value) = self.prev_ema {
            return Some(macd_value - self.signal.get_ema().unwrap());
        }
        None
    }
}
