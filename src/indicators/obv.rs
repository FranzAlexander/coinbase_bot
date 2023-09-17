pub struct Obv {
    pub obv: f64,
    pub prev_close: Option<f64>,
}

impl Obv {
    pub fn new() -> Self {
        Obv {
            obv: 0.0,
            prev_close: None,
        }
    }

    pub fn update(&mut self, new_close: f64, new_volume: f64) {
        if let Some(prev_close) = self.prev_close {
            if new_close > prev_close {
                self.obv += new_volume;
            } else if new_close < prev_close {
                self.obv -= new_volume;
            }
            // If new_close == prev_close, obv remains the same
        } else {
            // If this is the first entry, the OBV is just the volume of the day
            self.obv = new_volume
        }

        self.prev_close = Some(new_close);
    }
}