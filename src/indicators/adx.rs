use rust_decimal::{
    prelude::{FromPrimitive, Zero},
    Decimal,
};
use rust_decimal_macros::dec;
use std::collections::VecDeque;

pub struct Adx {
    period: usize,
    prev_close: Option<Decimal>,
    prev_high: Option<Decimal>,
    prev_low: Option<Decimal>,
    tr_values: VecDeque<Decimal>,
    positive_dm_values: VecDeque<Decimal>,
    negative_dm_values: VecDeque<Decimal>,
    smoothed_tr: Option<Decimal>,
    smoothed_positive_dm: Option<Decimal>,
    smoothed_negative_dm: Option<Decimal>,
    dx_values: VecDeque<Decimal>,
    adx: Option<Decimal>,
}

impl Adx {
    pub fn new(period: usize) -> Self {
        Adx {
            period,
            prev_close: None,
            prev_high: None,
            prev_low: None,
            tr_values: VecDeque::with_capacity(period),
            positive_dm_values: VecDeque::with_capacity(period),
            negative_dm_values: VecDeque::with_capacity(period),
            smoothed_tr: None,
            smoothed_positive_dm: None,
            smoothed_negative_dm: None,
            dx_values: VecDeque::with_capacity(period),
            adx: None,
        }
    }

    pub fn update(&mut self, current_high: Decimal, current_low: Decimal, current_close: Decimal) {
        let tr = self
            .true_range(current_high, current_low, current_close)
            .unwrap();
        let (positive_dm, negative_dm) = self.directional_movements(current_high, current_low);

        self.tr_values.push_back(tr);
        self.positive_dm_values
            .push_back(positive_dm.unwrap_or_else(Decimal::zero));
        self.negative_dm_values
            .push_back(negative_dm.unwrap_or_else(Decimal::zero));

        if self.tr_values.len() > self.period {
            self.tr_values.pop_front();
            self.positive_dm_values.pop_front();
            self.negative_dm_values.pop_front();
        }

        self.smoothed_tr = Some(self.smooth(self.smoothed_tr, &self.tr_values));
        self.smoothed_positive_dm =
            Some(self.smooth(self.smoothed_positive_dm, &self.positive_dm_values));
        self.smoothed_negative_dm =
            Some(self.smooth(self.smoothed_negative_dm, &self.negative_dm_values));

        let positive_di = self.smoothed_positive_dm.unwrap_or_else(Decimal::zero)
            * Decimal::from_usize(100).unwrap()
            / self.smoothed_tr.unwrap_or_else(Decimal::zero);
        let negative_di = self.smoothed_negative_dm.unwrap_or_else(Decimal::zero)
            * Decimal::from_usize(100).unwrap()
            / self.smoothed_tr.unwrap_or_else(Decimal::zero);

        let dx = dec!(100.0) * (positive_di - negative_di).abs() / (positive_di + negative_di);
        self.dx_values.push_back(dx);

        if self.dx_values.len() > self.period {
            self.dx_values.pop_front();
        }

        self.adx = Some(self.smooth(self.adx, &self.dx_values));

        // Update previous values at the end of the update method
        self.prev_close = Some(current_close);
        self.prev_high = Some(current_high);
        self.prev_low = Some(current_low);
    }

    fn true_range(
        &mut self,
        current_high: Decimal,
        current_low: Decimal,
        current_close: Decimal,
    ) -> Option<Decimal> {
        if let Some(prev_close) = self.prev_close {
            let range1 = current_high - current_low;
            let range2 = (current_high - prev_close).abs();
            let range3 = (current_low - prev_close).abs();
            Some(range1.max(range2).max(range3))
        } else {
            self.prev_close = Some(current_close);
            None
        }
    }

    fn directional_movements(
        &mut self,
        current_high: Decimal,
        current_low: Decimal,
    ) -> (Option<Decimal>, Option<Decimal>) {
        if let (Some(prev_high), Some(prev_low)) = (self.prev_high, self.prev_low) {
            let up_move = current_high - prev_high;
            let down_move = prev_low - current_low;

            if up_move > down_move && up_move > Decimal::zero() {
                (Some(up_move), None)
            } else if down_move > up_move && down_move > Decimal::zero() {
                (None, Some(down_move))
            } else {
                (None, None)
            }
        } else {
            self.prev_high = Some(current_high);
            self.prev_low = Some(current_low);
            (None, None)
        }
    }

    fn smooth(&self, prev_smoothed: Option<Decimal>, values: &VecDeque<Decimal>) -> Decimal {
        if let Some(prev) = prev_smoothed {
            prev - (prev / Decimal::from_usize(self.period).unwrap())
                + values.back().unwrap_or(&Decimal::zero()).clone()
        } else if values.len() == self.period {
            values.iter().cloned().sum()
        } else {
            Decimal::zero()
        }
    }

    pub fn get_adx(&self) -> Option<Decimal> {
        self.adx
    }
}
