#[derive(Debug)]
pub struct BollingerBands {
    period: usize,
    sma: Vec<f64>,
    prices: Vec<f64>,
    upper_band: Option<f64>,
    middle_band: Option<f64>,
    lower_band: Option<f64>,
}

impl BollingerBands {
    pub fn new(period: usize) -> Self {
        BollingerBands {
            period,
            sma: Vec::with_capacity(period),
            prices: Vec::new(),
            upper_band: None,
            middle_band: None,
            lower_band: None,
        }
    }

    pub fn update(&mut self, price: f64) {
        self.prices.push(price);
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }

        if self.prices.len() < self.period {
            return;
        }

        let sma_value: f64 = self.prices.iter().sum::<f64>() / self.period as f64;
        let variance: f64 = self
            .prices
            .iter()
            .map(|&p| (p - sma_value).powi(2))
            .sum::<f64>()
            / self.period as f64;
        let std_deviation: f64 = variance.sqrt();

        let upper_band = sma_value + (2.0 * std_deviation);
        let lower_band = sma_value - (2.0 * std_deviation);

        self.upper_band = Some(upper_band);
        self.middle_band = Some(sma_value);
        self.lower_band = Some(lower_band);
    }

    pub fn get_bands(&self) -> (Option<f64>, Option<f64>, Option<f64>) {
        (self.upper_band, self.middle_band, self.lower_band)
    }
}
