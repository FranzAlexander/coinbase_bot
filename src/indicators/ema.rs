use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;

pub struct Ema {
    period: usize,
    alpha: Decimal,
    values: Vec<Decimal>,
    prev_ema: Option<Decimal>,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        let alpha = Decimal::from_f64(2.0 / (period as f64 + 1.0)).unwrap();
        Self {
            period,
            alpha,
            values: Vec::with_capacity(period),
            prev_ema: None,
        }
    }

    pub fn update(&mut self, value: Decimal) {
        if self.values.len() < self.period {
            self.values.push(value);
            return;
        }

        let ema = if let Some(prev_ema) = self.prev_ema {
            (value * self.alpha) + prev_ema * (dec!(1.0) - self.alpha)
        } else {
            self.values.iter().sum::<Decimal>() / Decimal::from_usize(self.period).unwrap()
        };

        self.prev_ema = Some(ema);
    }

    pub fn get_current_ema(&self) -> Option<Decimal> {
        self.prev_ema
    }
}
