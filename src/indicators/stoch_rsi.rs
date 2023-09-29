// use std::collections::VecDeque;

// use super::rsi::Rsi;

// pub struct StochRsi {
//     rsi: Rsi,
//     period: usize,
//     k_period: usize,
//     d_period: usize,
//     stoch_rsis: VecDeque<f64>,
//     k_values: VecDeque<f64>,
//     pub k: f64, // To store the latest value of K
//     pub d: f64, // To store the latest value of D
// }

// impl StochRsi {
//     pub fn new(rsi_period: usize, period: usize, k_period: usize, d_period: usize) -> Self {
//         StochRsi {
//             rsi: Rsi::new(rsi_period),
//             period,
//             k_period,
//             d_period,
//             stoch_rsis: VecDeque::with_capacity(period),
//             k_values: VecDeque::with_capacity(k_period),
//             k: 0.0, // Initialized with 0.0
//             d: 0.0, // Initialized with 0.0
//         }
//     }

//     pub fn update(&mut self, close: f64, prev_price: f64) {
//         if let Some(rsi_val) = self.rsi.next(close) {
//             println!("RSI: {}", rsi_val);
//             self.stoch_rsis.push_back(rsi_val);
//             if self.stoch_rsis.len() > self.period {
//                 self.stoch_rsis.pop_front();
//             }

//             if self.stoch_rsis.len() == self.period {
//                 let min_rsi = self.stoch_rsis.iter().fold(f64::INFINITY, |a, &b| a.min(b));
//                 let max_rsi = self
//                     .stoch_rsis
//                     .iter()
//                     .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

//                 // Check to prevent division by zero which leads to NaN
//                 let stoch_rsi = if (max_rsi - min_rsi).abs() > f64::EPSILON {
//                     (rsi_val - min_rsi) / (max_rsi - min_rsi) * 100.0
//                 } else {
//                     50.0 // Mid-range if max and min RSI are the same
//                 };

//                 self.k_values.push_back(stoch_rsi);
//                 if self.k_values.len() > self.k_period {
//                     self.k_values.pop_front();
//                 }

//                 if self.k_values.len() == self.k_period {
//                     self.k = self.k_values.iter().sum::<f64>() / self.k_period as f64;
//                     self.d = self.k_values.iter().take(self.d_period).sum::<f64>()
//                         / self.d_period as f64;
//                 }
//             }
//         }
//     }
// }
#[derive(Debug)]
pub struct StochRSI {
    rsi: RSI,
    period: usize,
    rsi_history: VecDeque<f64>,
}

impl StochRSI {
    pub fn new(period: usize) -> Self {
        StochRSI {
            rsi: RSI::new(period),
            period,
            rsi_history: VecDeque::with_capacity(period),
        }
    }

    pub fn next(&mut self, current_price: f64) -> Option<f64> {
        if let Some(rsi_value) = self.rsi.next(current_price) {
            if self.rsi_history.len() == self.period {
                self.rsi_history.pop_front();
            }
            self.rsi_history.push_back(rsi_value);

            if self.rsi_history.len() == self.period {
                let rsi_low = *self.rsi_history.iter().min().unwrap_or(&100.0);
                let rsi_high = *self.rsi_history.iter().max().unwrap_or(&0.0);
                return Some(100.0 * (rsi_value - rsi_low) / (rsi_high - rsi_low));
            }
        }
        None
    }
}
