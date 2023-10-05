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

#[derive(Debug)]
pub struct Coin {
    pub balance: f64,
    pub active_trade: bool,
    pub stop_loss: f64,
    pub last_high: f64,
}

impl Coin {
    pub fn new(balance: f64, active_trade: bool, stop_loss: f64, last_high: f64) -> Self {
        Coin {
            balance,
            active_trade,
            stop_loss,
            last_high,
        }
    }

    pub fn update_coin(&mut self, active_trade: bool, stop_loss: f64, last_high: f64) {
        self.active_trade = active_trade;
        self.stop_loss = stop_loss;
        self.last_high = last_high;
    }
}
