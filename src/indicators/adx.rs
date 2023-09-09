pub struct Adx {
    period: usize,
    prev_close: Option<f64>,
    prev_high: f64,
    prev_low: f64,
    tr: f64,
    pos_dm: f64,
    neg_dm: f64,
    atr: f64,
    pos_di: f64,
    neg_di: f64,
    adx: f64,
}

impl Adx {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            prev_close: None,
            prev_high: 0_f64,
            prev_low: 0_f64,
            tr: 0_f64,
            pos_dm: 0.0,
            neg_dm: 0.0,
            atr: 0.0,
            pos_di: 0.0,
            neg_di: 0.0,
            adx: 0.0,
        }
    }

    pub fn update(&mut self, current_high: f64, current_low: f64, current_close: f64) {
        let tr = self.true_range(&current_high, &current_low);
        let (pd, nd) = self.direction_movement(&current_high, &current_low);

        if self.prev_close.is_none() {
            self.initialize(tr, pd, nd, current_close);
        } else {
            self.tr = ((self.tr * (self.period as f64 - 1.0_f64)) + tr) / self.period as f64;
            self.pos_dm =
                ((self.pos_dm * (self.period as f64 - 1.0_f64)) + pd) / self.period as f64;
            self.neg_dm =
                ((self.neg_dm * (self.period as f64 - 1.0_f64)) + nd) / self.period as f64;
        }

        self.atr = self.tr;
        self.pos_di = (self.pos_dm / self.atr) * 100.0_f64;
        self.neg_di = (self.neg_dm / self.atr) * 100.0_f64;

        let dx = ((self.pos_di - self.neg_di).abs()) / (self.pos_di + self.neg_di) * 100.0_f64;
        self.calculate_adx(dx);

        // Update previous values
        self.prev_high = current_high;
        self.prev_low = current_low;
        self.prev_close = Some(current_close);
    }

    fn true_range(&self, current_high: &f64, current_low: &f64) -> f64 {
        if let Some(prev_close) = self.prev_close {
            let high_low = current_high - current_low;
            let high_close = (current_high - prev_close).abs();
            let low_close = (current_low - prev_close).abs();

            high_low.max(high_close).max(low_close)
        } else {
            0.0_f64
        }
    }

    fn direction_movement(&self, current_high: &f64, current_low: &f64) -> (f64, f64) {
        let high_diff = current_high - self.prev_high;
        let low_diff = self.prev_low - current_low;

        let mut h_diff = 0.0;
        let mut l_diff = 0.0;

        if high_diff > 0.0 && high_diff > low_diff {
            h_diff = high_diff;
        }

        if low_diff > 0.0 && low_diff > high_diff {
            l_diff = low_diff;
        }

        (h_diff, l_diff)
    }

    fn initialize(&mut self, tr: f64, pd: f64, nd: f64, current_close: f64) {
        self.tr = tr;
        self.pos_dm = pd;
        self.neg_dm = nd;
        self.prev_close = Some(current_close);
    }

    fn calculate_adx(&mut self, dx: f64) {
        if self.adx == 0.0_f64 {
            self.adx = dx;
        } else {
            // Apply Wilder's Smoothing for ADX
            self.adx = ((self.adx * (self.period as f64 - 1.0_f64)) + dx) / self.period as f64;
        }
    }

    pub fn get_adx(&self) -> f64 {
        self.adx
    }
}
