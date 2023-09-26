use std::collections::VecDeque;

use super::rsi::Rsi;

pub struct StochRsi {
    rsi: Rsi,
    period_stoch: usize,
    smooth_k: usize,
    smooth_d: usize,
    rsis: VecDeque<f64>,
    stoch_rsis: VecDeque<f64>,
    avg_k: Option<f64>,
    avg_d: Option<f64>,
}

impl StochRsi {
    pub fn new(period_rsi: usize, period_stoch: usize, smooth_k: usize, smooth_d: usize) -> Self {
        StochRsi {
            rsi: Rsi::new(period_rsi),
            period_stoch,
            smooth_k,
            smooth_d,
            rsis: VecDeque::with_capacity(period_stoch),
            stoch_rsis: VecDeque::with_capacity(smooth_k),
            avg_k: None,
            avg_d: None,
        }
    }

    pub fn update(&mut self, current_close: f64, prev_close: f64) {
        if let Some(rsi_value) = self.rsi.update(current_close, prev_close) {
            self.rsis.push_back(rsi_value);
            if self.rsis.len() > self.period_stoch {
                self.rsis.pop_front();
            }

            if self.rsis.len() == self.period_stoch {
                let lowest_rsi = *self
                    .rsis
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let highest_rsi = *self
                    .rsis
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();

                let stoch_rsi_value = (rsi_value - lowest_rsi) / (highest_rsi - lowest_rsi);
                self.stoch_rsis.push_back(stoch_rsi_value);

                if self.stoch_rsis.len() > self.smooth_k {
                    self.stoch_rsis.pop_front();
                }

                if self.stoch_rsis.len() == self.smooth_k {
                    let avg_k_value = self.stoch_rsis.iter().sum::<f64>() / self.smooth_k as f64;
                    let avg_d_value = self
                        .stoch_rsis
                        .iter()
                        .rev()
                        .take(self.smooth_d)
                        .sum::<f64>()
                        / self.smooth_d as f64;

                    self.avg_k = Some(avg_k_value);
                    self.avg_d = Some(avg_d_value);

                }
            }
        }
        
    }

    pub fn get_avg_k(&self) -> Option<f64> {
        self.avg_k
    }

    pub fn get_avg_d(&self) -> Option<f64> {
        self.avg_d
    }
}
