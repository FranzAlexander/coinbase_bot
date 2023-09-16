// #[derive(Debug)]
// pub struct Rsi {
//     period: usize,
//     k: f64,
//     prev_price: Option<f64>,
//     avg_gain: Option<f64>,
//     avg_loss: Option<f64>,
//     rsi: Option<f64>,
// }

#[derive(Debug)]
pub struct Rsi {
    period: usize,
    prev_price: Option<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    rsi: Option<f64>,
}

impl Rsi {
    pub fn new(period: usize) -> Self {
        Rsi {
            period,
            prev_price: None,
            avg_gain: None,
            avg_loss: None,
            rsi: None,
        }
    }

    pub fn update(&mut self, price: f64) {
        if let Some(prev_price) = self.prev_price {
            let gain = if price > prev_price {
                price - prev_price
            } else {
                0.0
            };
            let loss = if price < prev_price {
                prev_price - price
            } else {
                0.0
            };

            if self.avg_gain.is_none() && self.avg_loss.is_none() {
                self.avg_gain = Some(gain);
                self.avg_loss = Some(loss);
            } else {
                self.avg_gain = Some(
                    (self.avg_gain.unwrap() * (self.period as f64 - 1.0) + gain)
                        / self.period as f64,
                );
                self.avg_loss = Some(
                    (self.avg_loss.unwrap() * (self.period as f64 - 1.0) + loss)
                        / self.period as f64,
                );
            }

            if let (Some(avg_gain), Some(avg_loss)) = (self.avg_gain, self.avg_loss) {
                let rs = avg_gain / avg_loss;
                let rsi = 100.0 - (100.0 / (1.0 + rs));
                self.prev_price = Some(price);

                self.rsi = Some(rsi);
            }
        }

        self.prev_price = Some(price);
    }

    // pub fn new(period: usize) -> Self {
    //     Rsi {
    //         period,
    //         k: 2.0 / (period + 1) as f64,
    //         prev_price: None,
    //         avg_gain: None,
    //         avg_loss: None,
    //         rsi: None,
    //     }
    // }

    // pub fn update(&mut self, close_price: f64) {
    //     let mut gain = 0.0;
    //     let mut loss = 0.0;

    //     if let Some(prev_price) = self.prev_price {
    //         if close_price > prev_price {
    //             gain = close_price - prev_price;
    //         } else {
    //             loss = prev_price - close_price;
    //         }
    //     } else {
    //         gain = 0.1;
    //         loss = 0.1;
    //     }

    //     self.prev_price = Some(close_price);

    //     if let Some(avg_gain) = self.avg_gain {
    //         self.avg_gain = Some(self.k * close_price + (1.0 - self.k) * avg_gain);
    //     } else {
    //         self.avg_gain = Some(gain);
    //     }

    //     if let Some(avg_loss) = self.avg_loss {
    //         self.avg_loss = Some(self.k * close_price + (1.0 - self.k) * avg_loss);
    //     } else {
    //         self.avg_loss = Some(loss);
    //     }

    //     self.rsi =
    //         Some(100.0 * self.avg_gain.unwrap() / (self.avg_gain.unwrap() / self.avg_loss.unwrap()))
    // }

    //     let (gain, loss) = self.calculate_gain_loss(prev_price, close_price);

    //     if self.count < self.period {
    //         self.avg_gain += gain;
    //         self.avg_loss += loss;
    //     } else {
    //         self.avg_gain =
    //             (self.avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
    //         self.avg_loss =
    //             (self.avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;
    //     }
    //     self.count += 1;

    //     if self.count >= self.period {
    //         let rs = self.avg_gain / self.avg_loss;
    //         let rsi = 100.0 - (100.0 / (1.0 + rs));
    //         self.rsi = Some(rsi);
    //     }
    // }

    fn calculate_gain_loss(&mut self, prev_price: f64, close_price: f64) -> (f64, f64) {
        let price_diff = close_price - prev_price;
        let gain = if price_diff > 0.0_f64 {
            price_diff
        } else {
            0.0_f64
        };
        let loss = if price_diff < 0.0_f64 {
            price_diff.abs()
        } else {
            0.0
        };

        (gain, loss)
    }

    pub fn get_rsi(&self) -> Option<f64> {
        self.rsi
    }
}

// impl Rsi {
//     pub fn new(period: usize) -> Self {
//         Self {
//             period,
//             count: 0,
//             prev_price: None,
//             avg_gain: Decimal::zero(),
//             avg_loss: Decimal::zero(),
//             value: Decimal::zero(),
//         }
//     }

//     pub fn update(&mut self, close_price: Decimal) {
//         if let Some(prev) = self.prev_price {
//             let (gain, loss) = self.calculate_gain_loss(prev, close_price);

//             if self.count < self.period {
//                 self.update_initial_avg_gain_loss(gain, loss);
//                 self.count += 1;
//             } else {
//                 if self.count == self.period {
//                     // This transforms the initial average to be correctly based on the period.
//                     self.avg_gain *= Decimal::from(self.period);
//                     self.avg_loss *= Decimal::from(self.period);
//                 }

//                 self.calculate_rsi(gain, loss);
//                 self.count += 1;
//             }
//         } else {
//             self.count += 1;
//         }

//         self.prev_price = Some(close_price);
//     }

//     fn calculate_gain_loss(&self, prev: Decimal, close_price: Decimal) -> (Decimal, Decimal) {
//         let gain = if close_price > prev {
//             close_price - prev
//         } else {
//             Decimal::zero()
//         };
//         let loss = if close_price < prev {
//             prev - close_price
//         } else {
//             Decimal::zero()
//         };
//         (gain, loss)
//     }

//     fn update_initial_avg_gain_loss(&mut self, gain: Decimal, loss: Decimal) {
//         self.avg_gain =
//             (self.avg_gain * Decimal::from(self.count) + gain) / Decimal::from(self.count + 1);
//         self.avg_loss =
//             (self.avg_loss * Decimal::from(self.count) + loss) / Decimal::from(self.count + 1);
//     }

//     fn calculate_rsi(&mut self, gain: Decimal, loss: Decimal) {
//         self.avg_gain =
//             (self.avg_gain * Decimal::from(self.period - 1) + gain) / Decimal::from(self.period);
//         self.avg_loss =
//             (self.avg_loss * Decimal::from(self.period - 1) + loss) / Decimal::from(self.period);

//         // Check for zero loss to avoid division by zero.
//         if self.avg_loss.is_zero() {
//             self.value = dec!(100);
//             return;
//         }

//         let rs = self.avg_gain / self.avg_loss;

//         self.value = dec!(100) - (dec!(100) / (Decimal::one() + rs));
//     }

//     pub fn get_rsi(&self) -> Decimal {
//         self.value
//     }
// }
