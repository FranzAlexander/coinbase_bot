use crate::model::account::Balance;

#[derive(Debug)]
pub struct Coin {
    pub balance: Balance,
    pub active_trade: bool,
    pub min_profit_percentage: f64,
    pub rolling_stop_loss: f64,
}

impl Coin {
    pub fn new(balance: Balance) -> Self {
        Coin {
            balance,
            active_trade: false,
            min_profit_percentage: 0.0,
            rolling_stop_loss: 0.0,
        }
    }
}
