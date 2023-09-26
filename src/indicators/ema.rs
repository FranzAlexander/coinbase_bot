use std::fmt;

#[derive(Debug)]
pub struct Ema {
    period: usize,
    multiplier: f64,
    prev_ema: Option<f64>,
    accumulated_sum: f64,
    count: usize,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        let multiplier = 2.0_f64 / (period as f64 + 1.0_f64);
        Ema {
            period,
            multiplier,
            prev_ema: None,
            accumulated_sum: 0_f64,
            count: 0,
        }
    }

    pub fn update(&mut self, new_price: f64) {
        self.count += 1;

        // If we don't have enough data for SMA yet, accumulate
        if self.count <= self.period {
            self.accumulated_sum += new_price;

            // If we've just reached enough data for SMA, compute it and set as initial EMA
            if self.count == self.period {
                let sma = self.accumulated_sum / self.period as f64;
                self.prev_ema = Some(sma);
            }
            return;
        }

        // From here on, we should have a prev_ema to work with
        let ema_value =
            ((new_price - self.prev_ema.unwrap()) * self.multiplier) + self.prev_ema.unwrap();
        self.prev_ema = Some(ema_value);
    }

    #[inline]
    pub fn get_ema(&self) -> Option<f64> {
        self.prev_ema
    }
}

impl fmt::Display for Ema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ema: {}", self.prev_ema.unwrap_or(0.0))
    }
}
