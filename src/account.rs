use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::Value;
use sha2::Sha256;
use tracing::{event, Level};
use uuid::Uuid;

use crate::{
    model::{
        account::{AccountList, ActiveTrade, Balance, Product},
        order::OrderResponse,
        OrderStatus,
    },
    util::{http_sign, send_get_request},
};

const ACCOUNT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";
const PRODUCT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/products";
const ORDER_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders";

const STOP_LOSS_PERCENTAGE: f64 = 0.002;
const BALANCE_CURRENCY: &str = "USD";
const COIN_CURRENCY: &str = "USDC";
const COIN_SYMBOL: &str = "XRP";

#[derive(Debug)]
pub struct BotAccount {
    client: reqwest::Client,
    balances: Vec<Balance>,
    api_key: String,
    secret_key: String,
    pub active_trade: Option<ActiveTrade>,
}

impl BotAccount {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");
        let secret_key = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

        let client = reqwest::Client::new();

        BotAccount {
            client,
            balances: Vec::new(),
            api_key,
            secret_key,
            active_trade: None,
        }
    }

    pub async fn update_balances(&mut self) {
        let accounts = self.get_wallet().await;

        for account in accounts.accounts.into_iter() {
            if let Some(pos) = self
                .balances
                .iter()
                .position(|x| x.currency == account.available_balance.currency)
            {
                self.balances[pos] = account.available_balance.clone();
            } else {
                self.balances.push(account.available_balance.clone());
            }
        }

        println!("{:?}", self.balances);
    }

    async fn get_wallet(&self) -> AccountList {
        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        // Generate the HMAC signature
        let signature = http_sign(
            self.secret_key.as_bytes(),
            &timestamp,
            "GET",
            "/api/v3/brokerage/accounts",
            "",
        );

        // Create headers
        let headers = self.create_headers(timestamp.as_str(), signature.as_str());

        send_get_request::<AccountList>(&self.client, ACCOUNT_API_URL, headers)
            .await
            .expect("Failed to send get request!")
    }

    pub async fn create_buy_order(&mut self) {
        let client_order_id = Uuid::new_v4().to_string();

        let amount = self.get_balance_value_by_currency(COIN_CURRENCY);

        let rounded_base_amount = format!("{:.2}", amount);

        let body = serde_json::json!({
                "client_order_id": client_order_id,
                "product_id":"XRP-USDC",
                "side": "BUY",
                "order_configuration":{
                   "market_market_ioc":{
                    "quote_size": rounded_base_amount
                   }
        }});

        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        let signature = http_sign(
            self.secret_key.as_bytes(),
            &timestamp,
            "POST",
            "/api/v3/brokerage/orders",
            &body.to_string(),
        );

        let headers = self.create_headers(&timestamp, &signature);

        let order = self
            .client
            .post(ORDER_API_URL)
            .headers(headers)
            .json(&body)
            .send()
            .await;

        match order {
            Ok(response) => {
                let order_response = response.json::<Value>().await;

                match order_response {
                    Ok(order) => {
                        let response: OrderResponse =
                            serde_json::from_value(order).expect("Failed to convert json");
                        if response.success {
                            event!(Level::INFO, "Successfuly brought: {}", COIN_SYMBOL);
                        }
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            Err(e) => {
                event!(Level::ERROR, "Sending Order: {:?}", e)
            }
        }
    }

    pub async fn create_sell_order(&mut self) {
        let client_order_id = self.active_trade.as_ref().unwrap().client_order_id;

        let base_amount = self.get_balance_value_by_currency(COIN_SYMBOL); // Amount you want to sell
        let rounded_base_amount = format!("{:.6}", base_amount);

        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        let body = serde_json::json!({
                "client_order_id": client_order_id,
                "product_id":"XRP-USDC",
                "side": "SELL",
                "order_configuration":{
                   "market_market_ioc":{
                    "quote_size": rounded_base_amount
                   }
        }});

        let signature = http_sign(
            self.secret_key.as_bytes(),
            &timestamp,
            "POST",
            "/api/v3/brokerage/orders",
            &body.to_string(),
        );

        let headers = self.create_headers(&timestamp, &signature);

        let order = self
            .client
            .post(ORDER_API_URL)
            .headers(headers)
            .json(&body)
            .send()
            .await;

        match order {
            Ok(response) => {
                let order_response: OrderResponse = response.json().await.unwrap();

                if order_response.success {
                    self.active_trade = None;

                    event!(Level::INFO, "Successfuly Sold: {}", COIN_SYMBOL);
                }
            }
            Err(e) => {
                event!(Level::ERROR, "Sending Sell Order: {:?}", e)
            }
        }
    }

    pub async fn get_product(&self) -> Product {
        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        let signature = http_sign(
            self.secret_key.as_bytes(),
            &timestamp,
            "GET",
            "/api/v3/brokerage/products/XRP-USD",
            "",
        );

        let headers = self.create_headers(&timestamp, &signature);

        let url = format!("{}/{}-{}", PRODUCT_API_URL, COIN_SYMBOL, BALANCE_CURRENCY);

        send_get_request::<Product>(&self.client, &url, headers)
            .await
            .expect("Failed to send message")
    }

    fn create_headers(&self, timestamp: &str, signature: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(
            "CB-ACCESS-KEY",
            HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers.insert("CB-ACCESS-SIGN", HeaderValue::from_str(signature).unwrap());
        headers.insert(
            "CB-ACCESS-TIMESTAMP",
            HeaderValue::from_str(timestamp).unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        headers
    }

    pub fn update_active_trade_status(&mut self, status: OrderStatus) {
        if self.active_trade.is_some() {
            self.active_trade.as_mut().unwrap().status = status
        }
    }

    fn get_balance_value_by_currency(&self, currency: &str) -> f64 {
        self.balances
            .iter()
            .find(|x| x.currency == currency)
            .expect("Failed to find currency")
            .value
    }
}
