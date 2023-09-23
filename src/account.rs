use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::Value;
use sha2::Sha256;
use tracing::{event, info, Level};
use uuid::Uuid;

use crate::{
    model::{
        account::{AccountList, ActiveTrade, Balance, Product},
        order::OrderResponse,
        OrderStatus, TradeSide,
    },
    util::{http_sign, send_get_request},
};

const ACCOUNT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";
const PRODUCT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/products";
const ORDER_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders";

const STOP_LOSS_PERCENTAGE: f64 = 0.0002;
const BALANCE_CURRENCY: &str = "USD";
const COIN_CURRENCY: &str = "USDC";
const COIN_SYMBOL: &str = "XRP";

#[derive(Debug)]
pub struct BotAccount {
    client: reqwest::Client,
    balances: Vec<Balance>,
    api_key: String,
    secret_key: String,
    pub active_trade: ActiveTrade,
}

impl BotAccount {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");
        let secret_key = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

        let client = reqwest::Client::new();

        let active_trade = ActiveTrade::new();

        BotAccount {
            client,
            balances: Vec::new(),
            api_key,
            secret_key,
            active_trade,
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

    pub async fn create_order(&mut self, order_type: TradeSide) {
        let price = self.get_product().await;

        println!("TRADE: {:?}", self.active_trade);

        let (client_order_id, currency) = match order_type {
            TradeSide::Buy => (Uuid::new_v4().to_string(), COIN_CURRENCY),
            TradeSide::Sell => (self.active_trade.order_id.to_owned(), COIN_SYMBOL),
        };

        let amount = self.get_balance_value_by_currency(currency);
        let rounded_base_amount = match order_type {
            TradeSide::Buy => format!("{:.2}", (2.25_f64 * 100.0).floor() / 100.0),
            TradeSide::Sell => format!("{:.6}", (amount * 100.0).floor() / 100.0),
        };

        let body = serde_json::json!({
                "client_order_id": client_order_id,
                "product_id":"XRP-USDC",
                "side": order_type,
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
                let json_resposne = response
                    .json::<Value>()
                    .await
                    .expect("Failed to parse json!");

                let order_response = serde_json::from_value::<OrderResponse>(json_resposne);

                match order_response {
                    Ok(order) => {
                        if order.success {
                            if order_type == TradeSide::Sell {
                                self.active_trade.active = false;
                            }
                            if order_type == TradeSide::Buy {
                                if let Some(success_response) = order.success_response {
                                    println!("{:?}", success_response);
                                    self.active_trade.set(
                                        success_response.order_id,
                                        success_response.client_order_id,
                                        price.price,
                                        price.price * rounded_base_amount.parse::<f64>().unwrap(),
                                        price.price * (1.0 - STOP_LOSS_PERCENTAGE),
                                    );
                                }

                                info!("Buy Response: {:?}", self.active_trade);
                            }
                            event!(Level::INFO, "Successfully {}: {}", order_type, COIN_SYMBOL);
                        }
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            Err(e) => {
                event!(Level::ERROR, "Sending {} Order: {:?}", order_type, e);
            }
        }
    }

    pub async fn create_buy_order(&mut self) {
        let client_order_id = Uuid::new_v4().to_string();

        let amount = self.get_balance_value_by_currency(COIN_CURRENCY);
        let rounded_base_amount = format!("{:.2}", (2.0_f64 * 100.0).floor() / 100.0);

        let request_body =
            self.create_order_body(client_order_id, TradeSide::Buy, rounded_base_amount);

        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        let signature = http_sign(
            self.secret_key.as_bytes(),
            &timestamp,
            "POST",
            "/api/v3/brokerage/orders",
            &request_body.to_string(),
        );

        let headers = self.create_headers(&timestamp, &signature);

        let order: OrderResponse = self
            .client
            .post(ORDER_API_URL)
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .expect("Failed to send order!")
            .json()
            .await
            .expect("Failed to parse order response!");
    }

    pub fn create_order_body(
        &self,
        client_order_id: String,
        side: TradeSide,
        rounded_base_amount: String,
    ) -> Value {
        serde_json::json!({
            "client_order_id":client_order_id,
            "product_id":"XRP-USDC",
            "side":side,
            "order_configuration":{
                "market_market_ioc":{
                    "quote_size":  rounded_base_amount
                }
            }
        })
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

    fn get_balance_value_by_currency(&self, currency: &str) -> f64 {
        self.balances
            .iter()
            .find(|x| x.currency == currency)
            .expect("Failed to find currency")
            .value
    }
}
