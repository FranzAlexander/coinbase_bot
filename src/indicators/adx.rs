#[derive(Debug)]
pub struct Adx {
    period: usize,
    prev_close: Option<f64>,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    tr_sum: f64,
    pos_dm_sum: f64,
    neg_dm_sum: f64,
    prev_tr: Option<f64>,
    prev_pos_dm: Option<f64>,
    prev_neg_dm: Option<f64>,
    dx_buffer: Vec<f64>,
}

impl Adx {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            prev_close: None,
            prev_high: None,
            prev_low: None,
            tr_sum: 0.0,
            pos_dm_sum: 0.0,
            neg_dm_sum: 0.0,
            prev_tr: None,
            prev_pos_dm: None,
            prev_neg_dm: None,
            dx_buffer: Vec::with_capacity(period),
        }
    }

    pub fn update(&mut self, current_high: f64, current_low: f64, current_close: f64) {
        if let (Some(prev_high), Some(prev_low), Some(prev_close)) =
            (self.prev_high, self.prev_low, self.prev_close)
        {
            let (ps_dm, ng_dm) =
                self.directional_movements(current_high, current_low, prev_high, prev_low);
            let tr = self.true_range(current_high, current_low, prev_close);

            if self.dx_buffer.len() < self.period {
                self.tr_sum += tr;
                self.pos_dm_sum += ps_dm;
                self.neg_dm_sum += ng_dm;
            } else {
                let smoothed_tr = (self.prev_tr.unwrap_or(0.0) * (self.period as f64 - 1.0) + tr)
                    / self.period as f64;
                let smoothed_pos_dm =
                    (self.prev_pos_dm.unwrap_or(0.0) * (self.period as f64 - 1.0) + ps_dm)
                        / self.period as f64;
                let smoothed_neg_dm =
                    (self.prev_neg_dm.unwrap_or(0.0) * (self.period as f64 - 1.0) + ng_dm)
                        / self.period as f64;

                self.prev_tr = Some(smoothed_tr);
                self.prev_pos_dm = Some(smoothed_pos_dm);
                self.prev_neg_dm = Some(smoothed_neg_dm);

                let (pos_di, neg_di) =
                    self.calculate_di(smoothed_pos_dm, smoothed_neg_dm, smoothed_tr);
                let dx = self.calculate_dx(pos_di, neg_di);

                self.dx_buffer.push(dx);
                if self.dx_buffer.len() > self.period {
                    self.dx_buffer.remove(0);
                }
            }
        }

        self.prev_high = Some(current_high);
        self.prev_low = Some(current_low);
        self.prev_close = Some(current_close);
    }

    fn true_range(&self, current_high: f64, current_low: f64, prev_close: f64) -> f64 {
        [
            current_high - current_low,
            (current_high - prev_close).abs(),
            (current_low - prev_close).abs(),
        ]
        .iter()
        .cloned()
        .fold(f64::NAN, f64::max)
    }

    fn directional_movements(
        &self,
        current_high: f64,
        current_low: f64,
        prev_high: f64,
        prev_low: f64,
    ) -> (f64, f64) {
        let pos_dm = if (current_high - prev_high) > (prev_low - current_low) {
            current_high - prev_high
        } else {
            0.0
        };

        let neg_dm = if (prev_low - current_low) > (current_high - prev_high) {
            prev_low - current_low
        } else {
            0.0
        };

        (pos_dm, neg_dm)
    }

    fn calculate_di(&self, pos_dm: f64, neg_dm: f64, tr: f64) -> (f64, f64) {
        let pos_di = (pos_dm / tr) * 100.0;
        let neg_di = (neg_dm / tr) * 100.0;
        (pos_di, neg_di)
    }

    fn calculate_dx(&self, pos_di: f64, neg_di: f64) -> f64 {
        100.0 * (pos_di - neg_di).abs() / (pos_di + neg_di)
    }

    pub fn get_adx(&self) -> Option<f64> {
        if self.dx_buffer.len() == self.period {
            Some(self.dx_buffer.iter().sum::<f64>() / self.period as f64)
        } else {
            None
        }
    }
}
