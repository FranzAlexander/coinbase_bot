use crate::model::{L2Data, Level2Event};

#[derive(Debug)]
pub struct OrderBook {
    pub bids: Vec<L2Data>,
    pub asks: Vec<L2Data>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn process_side_update(&mut self, update: L2Data) {
        match update.side {
            crate::model::Side::Bid => {
                self.bids.retain(|x| x.price_level != update.price_level);

                if update.new_quantity > 0.0 {
                    self.bids.push(update);
                }

                self.bids
                    .sort_by(|a, b| a.price_level.partial_cmp(&b.price_level).unwrap());
            }
            crate::model::Side::Offer => {
                self.asks.retain(|x| x.price_level != update.price_level);

                if update.new_quantity > 0.0 {
                    self.asks.push(update);
                }

                self.asks
                    .sort_by(|a, b| a.price_level.partial_cmp(&b.price_level).unwrap());
            }
        }
    }

    pub fn identify_support_and_resistance(&self, threshold: f64) -> (f64, f64) {
        let strong_support = self.get_strong_level(&self.bids, threshold);
        let strong_resistance = self.get_strong_level(&self.asks, threshold);
        (strong_support, strong_resistance)
    }

    fn get_strong_level(&self, side_data: &Vec<L2Data>, threshold: f64) -> f64 {
        let mut cumulative_volume = 0.0;
        for data in side_data {
            cumulative_volume += data.new_quantity;
            if cumulative_volume >= threshold {
                return data.price_level;
            }
        }
        0.0
    }
}
