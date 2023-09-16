#[derive(Debug)]
pub struct Adx {
    period: usize,
    smoothing: usize,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_close: Option<f64>,
    tr: f64,
    plus_dm: f64,
    minus_dm: f64,
    adx: Option<f64>,
}

impl Adx {
    pub fn new(period: usize, smoothing: usize) -> Self {
        Adx {
            period,
            smoothing,
            prev_high: None,
            prev_low: None,
            prev_close: None,
            tr: 0.0,
            plus_dm: 0.0,
            minus_dm: 0.0,
            adx: None,
        }
    }

    pub fn update(&mut self, high: f64, low: f64, close: f64) {
        if let (Some(prev_high), Some(prev_low), Some(prev_close)) =
            (self.prev_high, self.prev_low, self.prev_close)
        {
            let tr_curr = [
                high - low,
                (high - prev_close).abs(),
                (low - prev_close).abs(),
            ]
            .iter()
            .cloned()
            .fold(0. / 0., f64::max);

            self.tr = (self.tr * (self.smoothing as f64 - 1.0) + tr_curr) / self.smoothing as f64;

            let up_move = high - prev_high;
            let down_move = prev_low - low;

            let plus_dm_curr = if up_move > down_move && up_move > 0.0 {
                up_move
            } else {
                0.0
            };

            let minus_dm_curr = if down_move > up_move && down_move > 0.0 {
                down_move
            } else {
                0.0
            };

            self.plus_dm = (self.plus_dm * (self.smoothing as f64 - 1.0) + plus_dm_curr)
                / self.smoothing as f64;
            self.minus_dm = (self.minus_dm * (self.smoothing as f64 - 1.0) + minus_dm_curr)
                / self.smoothing as f64;

            let plus_di = (self.plus_dm / self.tr) * 100.0;
            let minus_di = (self.minus_dm / self.tr) * 100.0;

            let dx = ((plus_di - minus_di).abs() / (plus_di + minus_di)) * 100.0;

            // If the ADX has already been initialized, apply the smoothing, otherwise set it to the DX
            if let Some(prev_adx) = self.adx {
                self.adx =
                    Some((prev_adx * (self.smoothing as f64 - 1.0) + dx) / self.smoothing as f64);
            } else if self.prev_high.is_some() {
                // This assumes that we wait for the first period before calculating ADX
                self.adx = Some(dx);
            }
        }

        self.prev_high = Some(high);
        self.prev_low = Some(low);
        self.prev_close = Some(close);
    }

    pub fn get_adx(&self) -> Option<f64> {
        self.adx
    }
}
