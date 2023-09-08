use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;

pub struct Ema {
    period: usize,
    alpha: Decimal,
    total: Decimal,
    count: usize,
    prev_ema: Option<Decimal>,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        let alpha = Decimal::from_f64(2.0 / (period as f64 + 1.0)).unwrap();
        Self {
            period,
            alpha,
            total: dec!(0.0),
            count: 0,
            prev_ema: None,
        }
    }

    pub fn update(&mut self, value: Decimal) {
        if self.count < self.period {
            self.total += value;
            self.count += 1;

            if self.count == self.period {
                self.prev_ema = Some(self.total / Decimal::from_usize(self.period).unwrap())
            }

            return;
        }

        if let Some(prev_ema) = self.prev_ema {
            let ema = (value * self.alpha) + prev_ema * (dec!(1.0) - self.alpha);
            self.prev_ema = Some(ema);
        }
    }

    pub fn get_current_ema(&self) -> Option<Decimal> {
        self.prev_ema
    }
}
