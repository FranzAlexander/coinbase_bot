use std::fmt;

#[derive(Debug)]
pub struct Ema {
    period: usize,
    multiplier: f64,
    current: Option<f64>,
    count: usize,
    initial_sum: f64,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        Ema {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current: None,
            count: 0,
            initial_sum: 0.0,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        if self.count < self.period {
            self.initial_sum += price;
            self.count += 1;

            if self.count == self.period {
                let first_ema = self.initial_sum / self.period as f64;
                self.current = Some(first_ema);
            }
        } else {
            let prev_ema = self.current.unwrap_or_default();
            let new_ema = (price - prev_ema) * self.multiplier + prev_ema;
            self.current = Some(new_ema);
        }

        self.current
    }

    pub fn get_ema(&self) -> Option<f64> {
        self.current
    }
}

impl fmt::Display for Ema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ema: {}", self.current.unwrap_or(0.0))
    }
}
