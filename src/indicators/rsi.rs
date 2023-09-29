// // use std::collections::VecDeque;

// // pub struct Rsi {
// //     period: usize,
// //     gains: VecDeque<f64>,
// //     losses: VecDeque<f64>,
// //     avg_gain: Option<f64>,
// //     avg_loss: Option<f64>,
// // }

// // impl Rsi {
// //     pub fn new(period: usize) -> Self {
// //         Rsi {
// //             period,
// //             gains: VecDeque::with_capacity(period),
// //             losses: VecDeque::with_capacity(period),
// //             avg_gain: None,
// //             avg_loss: None,
// //         }
// //     }

// //     pub fn update(&mut self, current_close: f64, prev_close: f64) -> Option<f64> {
// //         let change = current_close - prev_close;

// //         if change > 0.0 {
// //             self.gains.push_back(change);
// //             self.losses.push_back(0.0);
// //         } else {
// //             self.gains.push_back(0.0);
// //             self.losses.push_back(change.abs()); // loss is positive
// //         }

// //         if self.gains.len() > self.period {
// //             self.gains.pop_front();
// //             self.losses.pop_front();
// //         }

// //         if self.gains.len() < self.period {
// //             return None;
// //         }

// //         if self.avg_gain.is_none() && self.avg_loss.is_none() {
// //             self.avg_gain = Some(self.gains.iter().sum::<f64>() / self.period as f64);
// //             self.avg_loss = Some(self.losses.iter().sum::<f64>() / self.period as f64);
// //         } else {
// //             let prev_avg_gain = self.avg_gain.unwrap();
// //             let prev_avg_loss = self.avg_loss.unwrap();

// //             let current_gain = *self.gains.back().unwrap();
// //             let current_loss = *self.losses.back().unwrap();

// //             self.avg_gain = Some(
// //                 ((prev_avg_gain * (self.period - 1) as f64) + current_gain) / self.period as f64,
// //             );
// //             self.avg_loss = Some(
// //                 ((prev_avg_loss * (self.period - 1) as f64) + current_loss) / self.period as f64,
// //             );
// //         }

// //         let rs = self.avg_gain.unwrap() / self.avg_loss.unwrap();
// //         Some(100.0 - (100.0 / (1.0 + rs)))
// //     }
// // }

// use std::collections::VecDeque;

// pub struct Rsi {
//     gain: f64,
//     loss: f64,
//     prev_close: Option<f64>,
//     avg_gain: VecDeque<f64>,
//     avg_loss: VecDeque<f64>,
//     period: usize,
// }

// impl Rsi {
//     pub fn new(period: usize) -> Self {
//         Rsi {
//             gain: 0.0,
//             loss: 0.0,
//             prev_close: None,
//             avg_gain: VecDeque::with_capacity(period),
//             avg_loss: VecDeque::with_capacity(period),
//             period,
//         }
//     }

//     pub fn update(&mut self, close: f64) -> Option<f64> {
//         if let Some(prev_close) = self.prev_close {
//             if close > prev_close {
//                 self.gain = close - prev_close;
//                 self.loss = 0.0;
//             } else {
//                 self.loss = prev_close - close;
//                 self.gain = 0.0;
//             }

//             if self.avg_gain.len() >= self.period {
//                 self.avg_gain.pop_front();
//                 self.avg_loss.pop_front();
//             }

//             self.avg_gain.push_back(self.gain);
//             self.avg_loss.push_back(self.loss);

//             let avg_gain = self.avg_gain.iter().sum::<f64>() / self.avg_gain.len() as f64;
//             let avg_loss = self.avg_loss.iter().sum::<f64>() / self.avg_loss.len() as f64;

//             if avg_loss != 0.0 {
//                 let rs = avg_gain / avg_loss;
//                 return Some(100.0 - (100.0 / (1.0 + rs)));
//             } else {
//                 return Some(100.0);
//             }
//         }

//         self.prev_close = Some(close);
//         None
//     }
// }

// #[derive(Debug)]
// pub struct Rsi {
//     period: usize,
//     prev_avg_gain: f64,
//     prev_avg_loss: f64,
//     count: usize,
// }

// impl Rsi {
//     pub fn new(period: usize) -> Self {
//         Rsi  {
//             period,
//             prev_avg_gain: 0.0,
//             prev_avg_loss: 0.0,
//             count: 0,
//         }
//     }

//     pub fn next(&mut self, current_price: f64, prev_price: f64) -> Option<f64> {
//         let gain = if current_price > prev_price {
//             current_price - prev_price
//         } else {
//             0.0
//         };

//         let loss = if current_price < prev_price {
//             prev_price - current_price
//         } else {
//             0.0
//         };

//         if self.count < self.period {
//             self.prev_avg_gain += gain;
//             self.prev_avg_loss += loss;
//             self.count += 1;

//             if self.count == self.period {
//                 self.prev_avg_gain /= self.period as f64;
//                 self.prev_avg_loss /= self.period as f64;
//             }
//             None
//         } else {
//             let avg_gain =
//                 (self.prev_avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
//             let avg_loss =
//                 (self.prev_avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;

//             if avg_loss == 0.0 {
//                 return Some(100.0);
//             }

//             let rs = avg_gain / avg_loss;
//             let rsi = 100.0 - (100.0 / (1.0 + rs));

//             self.prev_avg_gain = avg_gain;
//             self.prev_avg_loss = avg_loss;

//             Some(rsi)
//         }
//     }
// }
#[derive(Debug)]
pub struct Rsi {
    period: usize,
    prev_avg_gain: f64,
    prev_avg_loss: f64,
    count: usize,
    last_price: Option<f64>,
}

impl Rsi {
    pub fn new(period: usize) -> Self {
        Rsi {
            period,
            prev_avg_gain: 0.0,
            prev_avg_loss: 0.0,
            count: 0,
            last_price: None,
        }
    }

    pub fn next(&mut self, current_price: f64) -> Option<f64> {
        if let Some(prev_price) = self.last_price {
            let gain = if current_price > prev_price {
                current_price - prev_price
            } else {
                0.0
            };

            let loss = if current_price < prev_price {
                prev_price - current_price
            } else {
                0.0
            };

            if self.count < self.period {
                self.prev_avg_gain += gain;
                self.prev_avg_loss += loss;
                self.count += 1;

                if self.count == self.period {
                    self.prev_avg_gain /= self.period as f64;
                    self.prev_avg_loss /= self.period as f64;
                }
                self.last_price = Some(current_price);
                None
            } else {
                let avg_gain =
                    (self.prev_avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
                let avg_loss =
                    (self.prev_avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;

                if avg_loss == 0.0 {
                    self.last_price = Some(current_price);
                    return Some(100.0);
                }

                let rs = avg_gain / avg_loss;
                let rsi = 100.0 - (100.0 / (1.0 + rs));

                self.prev_avg_gain = avg_gain;
                self.prev_avg_loss = avg_loss;
                self.last_price = Some(current_price);

                Some(rsi)
            }
        } else {
            self.last_price = Some(current_price);
            None
        }
    }
}
