use std::collections::VecDeque;

struct StochasticOscillator {
    period: usize,
    prices: VecDeque<f64>,
    stoch_values: VecDeque<(f64, f64)>,
}

impl StochasticOscillator {
    fn new(period: usize) -> Self {
        StochasticOscillator {
            period,
            prices: VecDeque::with_capacity(period),
            stoch_values: VecDeque::new(),
        }
    }

    fn update(&mut self, price: f64) -> Option<(f64, f64)> {
        if self.prices.len() == self.period {
            self.prices.pop_front();
        }

        self.prices.push_back(price);

        if self.prices.len() < self.period {
            return None;
        }

        let high = *self
            .prices
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap();
        let low = *self
            .prices
            .iter()
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap();

        let k = (price - low) / (high - low) * 100.0;

        let d = if self.stoch_values.len() < 3 {
            k // If we don't have enough values, just set %D to %K
        } else {
            let sum: f64 = self.stoch_values.iter().rev().take(3).map(|(k, _)| k).sum();
            sum / 3.0
        };

        let new_values = (k, d);
        self.stoch_values.push_back(new_values);

        Some(new_values)
    }
}

fn main() {
    let mut stoch = StochasticOscillator::new(5);

    let incoming_data = vec![
        45.0, 46.0, 47.0, 48.5, 50.5, 52.0, 53.0, 55.0, 57.0, 56.0, 55.5, 54.0,
    ];
    for price in incoming_data {
        if let Some((k, d)) = stoch.update(price) {
            println!("K: {:.2}, D: {:.2}", k, d);
        }
    }
}
