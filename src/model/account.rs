use serde::{Deserialize, Serialize};

use super::string_or_float;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountType {
    #[serde(rename = "ACCOUNT_TYPE_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "ACCOUNT_TYPE_CRYPTO")]
    Crypto,
    #[serde(rename = "ACCOUNT_TYPE_FIAT")]
    Fiat,
    #[serde(rename = "ACCOUNT_TYPE_VAULT")]
    Vault,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountList {
    pub accounts: Vec<Account>,
    pub has_next: bool,
    pub cursor: Option<String>,
    pub size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingleAccount {
    pub account: Account,
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
    pub product_id: String,
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub quote_min_size: f64,
    #[serde(with = "string_or_float")]
    pub quote_max_size: f64,
    #[serde(with = "string_or_float")]
    pub base_min_size: f64,
    #[serde(with = "string_or_float")]
    pub base_max_size: f64,
}
