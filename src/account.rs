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

type HmacSha256 = Hmac<Sha256>;

const ACCOUNT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";
const PRODUCT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/products";
const ORDER_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders";

const MARKUP_PERCENTAGE: f64 = 0.0002;

#[derive(Debug)]
pub struct BotAccount {
    client: reqwest::Client,
    asset: Option<Balance>,
    currency: Option<Balance>,
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
            asset: None,
            currency: None,
            api_key,
            secret_key,
            active_trade: None,
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

        let account = send_get_request::<AccountList>(&self.client, &ACCOUNT_API_URL, headers)
            .await
            .expect("Failed to send get request!");

        account
    }

    pub async fn create_buy_order(&mut self) {
        let client_order_id = Uuid::new_v4().to_string();

        let current_price = self.get_product().await.unwrap();

        let base_amount = self.currency.as_ref().unwrap().value / current_price.price;
        let rounded_base_amount = format!("{:.8}", base_amount);

        let desired_price = current_price.price * (1.0 + MARKUP_PERCENTAGE);
        let rounded_desired_price = format!("{:.8}", desired_price);

        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        let signature = http_sign(
            self.secret_key.as_bytes(),
            &timestamp,
            "POST",
            "/api/v3/brokerage/orders",
            "",
        );

        let headers = self.create_headers(&timestamp, &signature);

        let order = self
            .client
            .post(ORDER_API_URL)
            .headers(headers)
            .json(&serde_json::json!({
                "client_order_id": client_order_id,
                "product_id":"BTC-USD",
                "side": "BUY",
                "order_configuration":{
                    "limit_limit_gtc":{
                        "base_size": rounded_base_amount.to_string(),
                        "limit_price": rounded_desired_price.to_string(),
                        "post_only": true
                    }
                }
            }))
            .send()
            .await;

        match order {
            Ok(response) => {
                let order_response: OrderResponse = response.json().await.unwrap();

                if order_response.success {
                    self.active_trade = Some(ActiveTrade {
                        order_id: order_response.success_response.order_id,
                        client_order_id: order_response.success_response.client_order_id,
                        amount: order_response.limit_limit_gtc.base_size,
                        price: order_response.limit_limit_gtc.limit_price,
                        stop_loss: 0.0,
                        status: OrderStatus::Open,
                    });

                    self.currency.as_mut().unwrap().value -=
                        order_response.limit_limit_gtc.limit_price;

                    self.asset.as_mut().unwrap().value += order_response.limit_limit_gtc.base_size;

                    event!(
                        Level::INFO,
                        "Successful Buy! Price: {}, Amount: {}",
                        order_response.limit_limit_gtc.limit_price,
                        order_response.limit_limit_gtc.base_size
                    );
                }
            }
            Err(e) => {
                event!(Level::ERROR, "Sending Order: {:?}", e)
            }
        }
    }

    pub async fn create_sell_order(&mut self) {
        let client_order_id = self.active_trade.as_ref().unwrap().client_order_id;

        let current_price = self.get_product().await.unwrap();

        let base_amount = self.asset.as_ref().unwrap().value; // Amount you want to sell
        let rounded_base_amount = format!("{:.8}", base_amount);

        // And perhaps a different logic for the desired_price:
        let desired_price = current_price.price * (1.0 - MARKUP_PERCENTAGE); // Selling at a markdown for quick sale?
        let rounded_desired_price = format!("{:.8}", desired_price);

        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        let signature = http_sign(
            self.secret_key.as_bytes(),
            &timestamp,
            "POST",
            "/api/v3/brokerage/orders",
            "",
        );

        let headers = self.create_headers(&timestamp, &signature);

        let order = self
            .client
            .post(ORDER_API_URL)
            .headers(headers)
            .json(&serde_json::json!({
                "client_order_id": client_order_id,
                "product_id":"BTC-USD",
                "side": "SELL",
                "order_configuration":{
                    "limit_limit_gtc":{
                        "base_size": rounded_base_amount.to_string(),
                        "limit_price": rounded_desired_price.to_string(),
                        "post_only": true
                    }
                }
            }))
            .send()
            .await;

        match order {
            Ok(response) => {
                let order_response: OrderResponse = response.json().await.unwrap();

                if order_response.success {
                    self.active_trade = None;

                    self.currency.as_mut().unwrap().value +=
                        order_response.limit_limit_gtc.limit_price;

                    self.asset.as_mut().unwrap().value -= order_response.limit_limit_gtc.base_size;

                    event!(
                        Level::INFO,
                        "Successful Sell! Price: {}, Amount: {}",
                        order_response.limit_limit_gtc.limit_price,
                        order_response.limit_limit_gtc.base_size
                    );
                }
            }
            Err(e) => {
                event!(Level::ERROR, "Sending Sell Order: {:?}", e)
            }
        }
    }

    pub async fn get_product(&self) -> Option<Product> {
        if let Some(asset) = &self.asset {
            let timestamp = format!("{}", chrono::Utc::now().timestamp());

            let signature = http_sign(
                self.secret_key.as_bytes(),
                &timestamp,
                "GET",
                "/api/v3/brokerage/products/BTC-USD",
                "",
            );

            let headers = self.create_headers(&timestamp, &signature);

            let url = format!("{}/{}-{}", PRODUCT_API_URL, asset.currency, "USD");

            let res = send_get_request::<Product>(&self.client, &url, headers)
                .await
                .expect("Failed to send message");

            Some(res)
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

    pub fn update_active_trade_status(&mut self, status: OrderStatus) {
        if self.active_trade.is_some() {
            self.active_trade.as_mut().unwrap().status = status
        }
    }
}
