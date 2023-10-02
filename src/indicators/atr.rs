#[derive(Debug)]
pub struct Atr {
    period: usize,
    prev_close: Option<f64>,
    true_range_sum: f64,
    values: Vec<f64>,
    atr_value: Option<f64>,
}

impl Atr {
    pub fn new(period: usize) -> Self {
        Atr {
            period,
            prev_close: None,
            true_range_sum: 0.0,
            values: Vec::new(),
            atr_value: None,
        }
    }

    pub fn update(&mut self, high: f64, low: f64, close: f64) {
        let true_range: f64 = match self.prev_close {
            Some(prev_close) => {
                let high_less_low = high - low;
                let high_less_prev_close = (high - prev_close).abs();
                let low_less_prev_close = (low - prev_close).abs();

                high_less_low
                    .max(high_less_prev_close)
                    .max(low_less_prev_close)
            }
            None => high - low,
        };

        if self.values.len() < self.period {
            self.true_range_sum += true_range;
            self.values.push(true_range);
            self.prev_close = Some(close);
        } else {
            self.true_range_sum -= self.values.remove(0);
            self.true_range_sum += true_range;
            self.values.push(true_range);
            self.prev_close = Some(close);
            self.atr_value = Some(self.true_range_sum / self.period as f64);
        }
    }

    pub fn get_atr(&self) -> Option<f64> {
        self.atr_value
    }
}
