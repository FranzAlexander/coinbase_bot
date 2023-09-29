// use std::fmt;

// #[derive(Debug)]
// pub struct Ema {
//     period: usize,
//     multiplier: f64,
//     prev_ema: Option<f64>,
//     accumulated_sum: f64,
//     count: usize,
// }

// impl Ema {
//     pub fn new(period: usize) -> Self {
//         let multiplier = 2.0_f64 / (period as f64 + 1.0_f64);
//         Ema {
//             period,
//             multiplier,
//             prev_ema: None,
//             accumulated_sum: 0_f64,
//             count: 0,
//         }
//     }

//     pub fn update(&mut self, new_price: f64) {
//         self.count += 1;

//         // If we don't have enough data for SMA yet, accumulate
//         if self.count <= self.period {
//             self.accumulated_sum += new_price;

//             // If we've just reached enough data for SMA, compute it and set as initial EMA
//             if self.count == self.period {
//                 let sma = self.accumulated_sum / self.period as f64;
//                 self.prev_ema = Some(sma);
//             }
//             return;
//         }

//         // From here on, we should have a prev_ema to work with
//         let ema_value =
//             ((new_price - self.prev_ema.unwrap()) * self.multiplier) + self.prev_ema.unwrap();
//         self.prev_ema = Some(ema_value);
//     }

//     #[inline]
//     pub fn get_ema(&self) -> Option<f64> {
//         self.prev_ema
//     }
// }

use std::fmt;

#[derive(Debug)]
pub struct Ema {
    sma: Vec<f64>,
    period: usize,
    multiplier: f64,
    current: Option<f64>,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        Ema {
            sma: Vec::with_capacity(period),
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current: None,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        if self.sma.len() < self.period {
            self.sma.push(price);
        }

        self.current = match self.current {
            Some(prev_ema) => Some((price - prev_ema) * self.multiplier + prev_ema),
            None => {
                if self.sma.len() == self.period {
                    Some(self.sma.iter().sum::<f64>() / self.period as f64)
                } else {
                    None
                }
            }
        };
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
