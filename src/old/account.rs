use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::{json, Value};
use sha2::Sha256;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::model::{AccountList, Balance};

type HmacSha256 = Hmac<Sha256>;

const API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";

#[derive(Debug)]
pub struct Account {
    client: reqwest::Client,
    pub asset: Option<Balance>,
    pub currency: Option<Balance>,
    api_key: String,
    secret_key: String,
}

impl Account {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");
        let secret_key = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

        let client = reqwest::Client::new();

        Account {
            client,
            api_key,
            asset: None,
            currency: None,
            secret_key,
        }
    }

    pub async fn update_balances(&mut self) {
        let accounts = self.get_wallet().await;

        for account in accounts.accounts.into_iter() {
            if account.available_balance.currency == "USDC" {
                self.currency = Some(account.available_balance);
            } else if account.available_balance.currency == "XRP" {
                self.asset = Some(account.available_balance)
            }
        }
    }

    pub async fn get_wallet(&self) -> AccountList {
        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        // Generate the HMAC signature
        let signature = self.sign(&timestamp, "GET", "/api/v3/brokerage/accounts", "");

        // Create headers
        let mut headers = HeaderMap::new();
        headers.insert(
            "CB-ACCESS-KEY",
            HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers.insert("CB-ACCESS-SIGN", HeaderValue::from_str(&signature).unwrap());
        headers.insert(
            "CB-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&timestamp).unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Make the GET request with headers
        let response: Value = self
            .client
            .get(API_URL)
            .headers(headers)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let account: AccountList = serde_json::from_value(response).unwrap();
        account
    }

    pub fn can_buy(&self) -> bool {
        if let Some(currency) = &self.currency {
            currency.value > 1.0
        } else {
            false
        }
    }

    pub fn can_sell(&self) -> bool {
        if let Some(asset) = &self.asset {
            asset.value > 1.0
        } else {
            false
        }
    }

    fn sign(&self, timestamp: &str, method: &str, request_path: &str, body: &str) -> String {
        let message = format!("{}{}{}{}", timestamp, method, request_path, body);

        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        let result = mac.finalize();

        format!("{:x}", result.into_bytes())
    }
}
