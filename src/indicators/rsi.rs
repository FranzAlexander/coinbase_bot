#[derive(Debug)]
pub struct Rsi {
    period: usize,
    count: usize,
    prev_price: Option<f64>,
    avg_gain: f64,
    avg_loss: f64,
    current_rsi: Option<f64>, // This will store the current RSI value
}

impl Rsi {
    pub fn new(period: usize) -> Self {
        Rsi {
            period,
            count: 0,
            prev_price: None,
            avg_gain: 0.0,
            avg_loss: 0.0,
            current_rsi: None, // Initialize as None
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        if let Some(prev_price) = self.prev_price {
            let gain = (price - prev_price).max(0.0);
            let loss = (prev_price - price).max(0.0);

            if self.count < self.period {
                self.avg_gain += gain;
                self.avg_loss += loss;
                self.count += 1;

                if self.count == self.period {
                    self.avg_gain /= self.period as f64;
                    self.avg_loss /= self.period as f64;
                }
            } else {
                self.avg_gain =
                    (self.avg_gain * (self.period - 1) as f64 + gain) / self.period as f64;
                self.avg_loss =
                    (self.avg_loss * (self.period - 1) as f64 + loss) / self.period as f64;
            }
        }

        self.prev_price = Some(price);

        if self.count < self.period {
            None
        } else {
            let rs = if self.avg_loss == 0.0 {
                100.0
            } else {
                self.avg_gain / self.avg_loss
            };

            self.current_rsi = Some(100.0 - (100.0 / (1.0 + rs))); // Store the current RSI value
            self.current_rsi
        }
    }

    #[inline]
    // Method to retrieve the current RSI value
    pub fn get_current_rsi(&self) -> Option<f64> {
        self.current_rsi
    }
}
