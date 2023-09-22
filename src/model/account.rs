use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{string_or_float, OrderStatus};

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
    pub currency: String,
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
    pub order_id: Uuid,
    pub client_order_id: Uuid,
    pub price: f64,
    pub amount: f64,
    pub stop_loss: f64,
}

impl ActiveTrade {
    pub fn new() -> Self {
        ActiveTrade {
            active: false,
            order_id: Uuid::default(),
            client_order_id: Uuid::default(),
            price: 0.0,
            amount: 0.0,
            stop_loss: 0.0,
        }
    }

    pub fn set(
        &mut self,
        order_id: Uuid,
        client_order_id: Uuid,
        price: f64,
        amout: f64,
        stop_loss: f64,
    ) {
        self.active = true;
        self.order_id = order_id;
        self.client_order_id = client_order_id;
        self.price = price;
        self.amount = amout;
        self.stop_loss = stop_loss;
    }
}
