use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CoinSymbol {
    Usd,
    Usdc,
    Xrp,
    Ada,
    Link,
    Btc,
    Eth,
    #[serde(other)]
    Unknown,
}

#[derive(Debug)]
pub struct Coin {
    pub balance: f64,
    pub active_trade: bool,
    pub min_profit_percentage: f64,
    pub rolling_stop_loss: f64,
    pub rolling_stop_loss_first_hit: bool,
    pub hard_stop: f64,
}

impl Coin {
    pub fn new(balance: f64) -> Self {
        Coin {
            balance,
            active_trade: false,
            min_profit_percentage: 0.0,
            rolling_stop_loss: 0.0,
            rolling_stop_loss_first_hit: false,
            hard_stop: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.active_trade = false;
        self.min_profit_percentage = 0.0;
        self.rolling_stop_loss = 0.0;
        self.rolling_stop_loss_first_hit = false;
        self.hard_stop = 0.0;
    }
}

impl From<CoinSymbol> for String {
    fn from(value: CoinSymbol) -> Self {
        String::from(match value {
            CoinSymbol::Ada => "ADA",
            CoinSymbol::Link => "LINK",
            CoinSymbol::Usd => "USD",
            CoinSymbol::Usdc => "USDC",
            CoinSymbol::Xrp => "XRP",
            CoinSymbol::Btc => "BTC",
            CoinSymbol::Eth => "ETH",
            _ => "NA",
        })
    }
}

impl FromStr for CoinSymbol {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ADA" => Ok(CoinSymbol::Ada),
            "LINK" => Ok(CoinSymbol::Link),
            "USD" => Ok(CoinSymbol::Usd),
            "USDC" => Ok(CoinSymbol::Usdc),
            "XRP" => Ok(CoinSymbol::Xrp),
            "BTC" => Ok(CoinSymbol::Btc),
            "ETH" => Ok(CoinSymbol::Eth),
            _ => Err(()),
        }
    }
}
