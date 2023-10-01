use serde::{Deserialize, Serialize};

use crate::coin::CoinSymbol;

use super::string_or_float;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    AccountTypeUnspecified,
    AccountTypeCrypto,
    AccountTypeFiat,
    AccountTypeVault,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountList {
    pub accounts: Vec<Account>,
    pub has_next: bool,
    pub cursor: Option<String>,
    pub size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub uuid: String,
    pub name: String,
    pub currency: CoinSymbol,
    pub available_balance: Balance,
    pub default: bool,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub ready: bool,
    pub hold: Balance,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Balance {
    #[serde(with = "string_or_float")]
    pub value: f64,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
pub struct Product {
    #[serde(with = "string_or_float")]
    pub price: f64,
}

#[derive(Debug)]
pub struct ActiveTrade {
    pub active: bool,
    pub order_id: String,
    pub client_order_id: String,
    pub price: f64,
    pub amount: f64,
    pub stop_loss: f64,
}

impl ActiveTrade {
    pub fn new(
        active: bool,
        order_id: String,
        client_order_id: String,
        price: f64,
        amount: f64,
        stop_loss: f64,
    ) -> Self {
        ActiveTrade {
            active,
            order_id,
            client_order_id,
            price,
            amount,
            stop_loss,
        }
    }
}
