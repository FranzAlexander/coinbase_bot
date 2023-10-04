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
            values: Vec::with_capacity(period),
            atr_value: None,
        }
    }

    pub fn update(&mut self, high: f64, low: f64, close: f64) {
        let current_tr = if let Some(prev_close) = self.prev_close {
            self.true_range(high, low, prev_close)
        } else {
            high - low
        };

        self.prev_close = Some(close);
        self.values.push(current_tr);

        if self.values.len() > self.period {
            self.values.remove(0);
        }

        if self.values.len() == self.period {
            if self.atr_value.is_none() {
                self.atr_value = Some(self.values.iter().sum::<f64>() / self.period as f64);
            } else {
                let last_atr = self.atr_value.unwrap();
                let new_atr =
                    ((self.period - 1) as f64 * last_atr + current_tr) / self.period as f64;
                self.atr_value = Some(new_atr);
            }
        }
    }

    fn true_range(&self, high: f64, low: f64, prev_close: f64) -> f64 {
        let hl = high - low;
        let hc = (high - prev_close).abs();
        let lc = (low - prev_close).abs();

        hl.max(hc.max(lc))
    }

    #[inline]
    pub fn get_atr(&self) -> Option<f64> {
        self.atr_value
    }
}
