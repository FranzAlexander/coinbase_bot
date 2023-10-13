use super::rsi::Rsi;

pub struct StochRsi {
    rsi: Rsi,
    period: usize,   // for StochRSI
    d_period: usize, // for the D line
    rsi_values: Vec<f64>,
    k_values: Vec<f64>, // for storing past K line values
    current_stoch_rsi: Option<f64>,
    current_d: Option<f64>,
}

impl StochRsi {
    pub fn new(rsi_period: usize, stoch_rsi_period: usize, d_period: usize) -> Self {
        StochRsi {
            rsi: Rsi::new(rsi_period),
            period: stoch_rsi_period,
            d_period,
            rsi_values: Vec::with_capacity(stoch_rsi_period),
            k_values: Vec::with_capacity(d_period),
            current_stoch_rsi: None,
            current_d: None,
        }
    }

    pub fn update(&mut self, price: f64) -> (Option<f64>, Option<f64>) {
        if let Some(rsi_value) = self.rsi.update(price) {
            self.rsi_values.push(rsi_value);

            if self.rsi_values.len() > self.period {
                self.rsi_values.remove(0);
            }

            if self.rsi_values.len() == self.period {
                let min_rsi = self
                    .rsi_values
                    .iter()
                    .cloned()
                    .fold(f64::INFINITY, f64::min);
                let max_rsi = self
                    .rsi_values
                    .iter()
                    .cloned()
                    .fold(f64::NEG_INFINITY, f64::max);

                let k = (rsi_value - min_rsi) / (max_rsi - min_rsi);
                self.current_stoch_rsi = Some(k);
                self.k_values.push(k);

                if self.k_values.len() > self.d_period {
                    self.k_values.remove(0);
                }

                if self.k_values.len() == self.d_period {
                    let d: f64 = self.k_values.iter().sum::<f64>() / self.d_period as f64;
                    self.current_d = Some(d);
                }
            }
        }

        (self.current_stoch_rsi, self.current_d)
    }

    #[inline]
    pub fn get_current_k(&self) -> Option<f64> {
        self.current_stoch_rsi
    }

    #[inline]
    pub fn get_current_d(&self) -> Option<f64> {
        self.current_d
    }
}
