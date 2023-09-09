use super::ema::Ema;

pub struct Macd {
    signal: Ema,
    prev_macd: Option<f64>,
}

impl Macd {
    pub fn new(signal_period: usize) -> Self {
        Macd {
            signal: Ema::new(signal_period),
            prev_macd: None,
        }
    }

    pub fn update(&mut self, fast_ema_value: f64, slow_ema_value: f64) {
        let macd_value = fast_ema_value - slow_ema_value;
        self.prev_macd = Some(macd_value);
        self.signal.update(macd_value);
    }

    pub fn get_macd(&self) -> Option<f64> {
        self.prev_macd
    }

    pub fn get_signal(&self) -> Option<f64> {
        self.signal.get_ema()
    }

    pub fn get_histogram(&self) -> Option<f64> {
        if let Some(macd_value) = self.prev_macd {
            return Some(macd_value - self.signal.get_ema().unwrap_or(0.0));
        }
        None
    }
}
