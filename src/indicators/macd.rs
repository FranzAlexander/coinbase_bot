use rust_decimal::Decimal;

use super::ema::Ema;

pub struct Macd {
    short_ema: Ema,
    long_ema: Ema,
    signal_ema: Ema,
    current_macd: Option<Decimal>,
}

impl Macd {
    pub fn new(short_period: usize, long_period: usize, signal_period: usize) -> Self {
        let short_ema = Ema::new(short_period);
        let long_ema = Ema::new(long_period);
        let signal_ema = Ema::new(signal_period);

        Macd {
            short_ema,
            long_ema,
            signal_ema,
            current_macd: None,
        }
    }

    pub fn update(&mut self, value: Decimal) {
        self.short_ema.update(value);
        self.long_ema.update(value);

        if let (Some(short_val), Some(long_val)) = (
            self.short_ema.get_current_ema(),
            self.long_ema.get_current_ema(),
        ) {
            self.current_macd = Some(short_val - long_val);
            self.signal_ema.update(self.current_macd.unwrap());
        }
    }

    pub fn get_signal_n_macd(&self) -> (Option<Decimal>, Option<Decimal>) {
        (self.signal_ema.get_current_ema(), self.current_macd)
    }
}
