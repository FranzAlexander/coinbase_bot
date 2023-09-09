pub struct Ema {
    period: usize,
    k: f64,
    prev_ema: Option<f64>,
    accumulated_sum: f64,
    count: usize,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        let k = 2.0_f64 / (period as f64 + 1.0_f64);
        Ema {
            period,
            k,
            prev_ema: None,
            accumulated_sum: 0_f64,
            count: 0,
        }
    }

    pub fn update(&mut self, new_price: f64) {
        if self.count < self.period {
            self.accumulated_sum += new_price;
            self.count += 1;
            return;
        }

        if self.prev_ema.is_none() {
            let sma = self.accumulated_sum / self.period as f64;
            self.prev_ema = Some(sma);
        } else {
            let ema_value =
                ((new_price - self.prev_ema.unwrap()) * self.k) + self.prev_ema.unwrap();
            self.prev_ema = Some(ema_value);
        }
    }

    pub fn get_ema(&self) -> Option<f64> {
        self.prev_ema
    }
}
