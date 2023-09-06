use rust_decimal::{
    prelude::{One, Zero},
    Decimal,
};
use rust_decimal_macros::dec;

pub struct Rsi {
    period: usize,
    count: usize,
    prev_price: Option<Decimal>,
    avg_gain: Decimal,
    avg_loss: Decimal,
    value: Decimal,
}

impl Rsi {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            count: 0,
            prev_price: None,
            avg_gain: Decimal::zero(),
            avg_loss: Decimal::zero(),
            value: Decimal::zero(),
        }
    }

    pub fn update(&mut self, close_price: Decimal) {
        match self.prev_price {
            Some(prev) => {
                let (gain, loss) = self.caluclate_gain_loss(prev, close_price);

                self.update_avg_gain_loss(gain, loss);

                if self.count < self.period {
                    self.count += 1;
                } else {
                    self.caluclate_rsi(gain, loss);
                }
            }
            None => {
                self.count += 1;
            }
        }

        self.prev_price = Some(close_price);
    }

    fn caluclate_gain_loss(&self, prev: Decimal, close_price: Decimal) -> (Decimal, Decimal) {
        let gain = if close_price > prev {
            close_price - prev
        } else {
            Decimal::zero()
        };
        let loss = if close_price < prev {
            prev - close_price
        } else {
            Decimal::zero()
        };
        (gain, loss)
    }

    fn update_avg_gain_loss(&mut self, gain: Decimal, loss: Decimal) {
        self.avg_gain =
            (self.avg_gain * Decimal::from(self.count) + gain) / Decimal::from(self.count + 1);
        self.avg_loss =
            (self.avg_loss * Decimal::from(self.count) + loss) / Decimal::from(self.count + 1);
    }

    fn caluclate_rsi(&mut self, gain: Decimal, loss: Decimal) {
        self.avg_gain =
            (self.avg_gain * Decimal::from(self.period - 1) + gain) / Decimal::from(self.period);
        self.avg_loss =
            (self.avg_loss * Decimal::from(self.period - 1) + loss) / Decimal::from(self.period);

        // Check for zero loss to avoid division by zero.
        if self.avg_loss.is_zero() {
            self.value = dec!(100);
            return;
        }

        let rs = self.avg_gain / self.avg_loss;

        self.value = dec!(100) - (dec!(100) / (Decimal::one() + rs));
    }

    pub fn get_rsi(&self) -> Decimal {
        self.value
    }
}
