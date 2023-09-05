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

        let short_ema = self.short_ema.get_current_ema();
        let long_ema = self.long_ema.get_current_ema();

        if short_ema.is_some() && long_ema.is_some() {
            self.current_macd = Some(short_ema.unwrap() - long_ema.unwrap());
            self.signal_ema.update(self.current_macd.unwrap());
        } else {
            return;
        }
    }

    pub fn get_signal_n_macd(&self) -> (Option<Decimal>, Option<Decimal>) {
        (self.signal_ema.get_current_ema(), self.current_macd)
    }
}
