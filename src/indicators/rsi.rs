use std::collections::VecDeque;

pub struct Rsi {
    period: usize,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
}

impl Rsi {
    pub fn new(period: usize) -> Self {
        Rsi {
            period,
            gains: VecDeque::with_capacity(period),
            losses: VecDeque::with_capacity(period),
            avg_gain: None,
            avg_loss: None,
        }
    }

    pub fn update(&mut self, current_close: f64, prev_close: f64) -> Option<f64> {
        let change = current_close - prev_close;

        if change > 0.0 {
            self.gains.push_back(change);
            self.losses.push_back(0.0);
        } else {
            self.gains.push_back(0.0);
            self.losses.push_back(change.abs()); // loss is positive
        }

        if self.gains.len() > self.period {
            self.gains.pop_front();
            self.losses.pop_front();
        }

        if self.gains.len() < self.period {
            return None;
        }

        if self.avg_gain.is_none() && self.avg_loss.is_none() {
            self.avg_gain = Some(self.gains.iter().sum::<f64>() / self.period as f64);
            self.avg_loss = Some(self.losses.iter().sum::<f64>() / self.period as f64);
        } else {
            let prev_avg_gain = self.avg_gain.unwrap();
            let prev_avg_loss = self.avg_loss.unwrap();

            let current_gain = *self.gains.back().unwrap();
            let current_loss = *self.losses.back().unwrap();

            self.avg_gain = Some(
                ((prev_avg_gain * (self.period - 1) as f64) + current_gain) / self.period as f64,
            );
            self.avg_loss = Some(
                ((prev_avg_loss * (self.period - 1) as f64) + current_loss) / self.period as f64,
            );
        }

        let rs = self.avg_gain.unwrap() / self.avg_loss.unwrap();
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
}
