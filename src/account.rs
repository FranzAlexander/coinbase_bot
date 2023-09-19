use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::Value;
use sha2::Sha256;
use uuid::Uuid;

use crate::model::account::{AccountList, Balance, CurrentTrade, Product};

type HmacSha256 = Hmac<Sha256>;

const ACCOUNT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";
const PRODUCT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/products";
const ORDER_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders";

#[derive(Debug)]
pub struct BotAccount {
    client: reqwest::Client,
    pub asset: Option<Balance>,
    pub currency: Option<Balance>,
    api_key: String,
    secret_key: String,
    trade: Option<CurrentTrade>,
}

impl BotAccount {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");
        let secret_key = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

        let client = reqwest::Client::new();

        BotAccount {
            client,
            asset: None,
            currency: None,
            api_key,
            secret_key,
            trade: None,
        }
    }

    pub async fn update_balances(&mut self) {
        let accounts = self.get_wallet().await;

        for account in accounts.accounts.into_iter() {
            if account.available_balance.currency == "USDC" {
                self.currency = Some(account.available_balance);
            } else if account.available_balance.currency == "BTC" {
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
        let account: AccountList = self
            .client
            .get(ACCOUNT_API_URL)
            .headers(headers)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        account
    }

    pub async fn create_n_send_order(&self) {
        let client_order_id = Uuid::new_v4().to_string();
    }

    pub async fn get_product(&self) -> Option<Product> {
        if let Some(asset) = &self.asset {
            let timestamp = format!("{}", chrono::Utc::now().timestamp());

            let signature = self.sign(&timestamp, "GET", "/api/v3/brokerage/products/BTC-USD", "");

            let headers = self.create_headers(&timestamp, &signature);

            let url = format!("{}/{}-{}", PRODUCT_API_URL, asset.currency, "USD");
            let response: Product = self
                .client
                .get(&url)
                .headers(headers)
                .send()
                .await
                .expect("Error fetching the product")
                .json()
                .await
                .expect("Failed to parse response as json");

            Some(response)
        } else {
            None
        }
    }

    fn create_headers(&self, timestamp: &str, signature: &str) -> HeaderMap {
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

        headers
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
