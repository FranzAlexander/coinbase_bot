use std::{collections::HashMap, time::SystemTime};

use chrono::{Duration, Utc};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

use serde_json::Value;
use uuid::Uuid;

use crate::{
    coin::Coin,
    model::{
        account::{AccountList, Balance, Product},
        event::CandleEvent,
        order::{CurrentOrder, CurrentOrderResponse, OrderResponse},
        TradeSide,
    },
    trading_bot::TradeSignal,
    util::{create_headers, http_sign, send_get_request},
};

const ACCOUNT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";
const PRODUCT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/products";
const ORDER_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders";
const HISTORICAL_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders/historical";

const ACCOUNT_REQUEST_PATH: &str = "/api/v3/brokerage/accounts";
const ORDER_REQUEST_PATH: &str = "/api/v3/brokerage/orders";

const BALANCE_CURRENCY: &str = "USD";
const COIN_CURRENCY: &str = "USDC";
const COIN_SYMBOL: &str = "BTC";

pub const USD_SYMBOL: &str = "USD";
pub const USDC_SYMBOL: &str = "USDC";
pub const BTC_SYMBOL: &str = "BTC";
pub const XRP_SYMBOL: &str = "XRP";
pub const ETH_SYMBOL: &str = "ETH";
pub const ADA_SYMBOL: &str = "ADA";

#[derive(Debug)]
pub struct BotAccount {
    client: reqwest::Client,
    coins: HashMap<String, Coin>,
    api_key: String,
    secret_key: String,
    trade_active: bool,
}

impl BotAccount {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");
        let secret_key = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

        let client = reqwest::Client::new();
        let coins = HashMap::new();

        BotAccount {
            client,
            coins,
            api_key,
            secret_key,
            trade_active: false,
        }
    }

    pub async fn update_balances(&mut self) {
        let accounts = self.get_wallet().await;

        for account in accounts.accounts.into_iter() {
            match account.currency.as_str() {
                USDC_SYMBOL | BTC_SYMBOL | XRP_SYMBOL | ETH_SYMBOL | ADA_SYMBOL => {
                    self.coins
                        .insert(account.currency, Coin::new(account.available_balance.value));
                }
                _ => {}
            }
        }
    }

    async fn get_wallet(&self) -> AccountList {
        let headers = create_headers(
            self.secret_key.as_bytes(),
            &self.api_key,
            "GET",
            ACCOUNT_REQUEST_PATH,
            "",
        );

        send_get_request::<AccountList>(&self.client, ACCOUNT_API_URL, headers)
            .await
            .expect("Failed to send get request!")
    }

    pub async fn create_order(&mut self, order_type: TradeSide, symbol: String) {
        let client_order_id = Uuid::new_v4().to_string();

        let amount = if order_type == TradeSide::Buy {
            self.coins.get(USDC_SYMBOL).unwrap().balance
        } else {
            self.coins.get(&symbol).unwrap().balance
        };

        let (quote_size, base_size) = match order_type {
            TradeSide::Buy => (
                Some(format!("{:.2}", (5.0_f64 * 100.0).floor() / 100.0)),
                None,
            ),
            TradeSide::Sell => (None, Some(format!("{:.8}", amount))),
        };

        let product_id = format!("{symbol}-{USDC_SYMBOL}");

        let body = serde_json::json!({
            "client_order_id": client_order_id,
            "product_id":product_id,
            "side": order_type,
            "order_configuration":{
                "market_market_ioc":{
                    "quote_size":  quote_size,
                    "base_size":  base_size
                }
            }
        });

        let headers = create_headers(
            self.secret_key.as_bytes(),
            &self.api_key,
            "POST",
            ORDER_REQUEST_PATH,
            &body.to_string(),
        );

        let order: OrderResponse = self
            .client
            .post(ORDER_API_URL)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .expect("Failed to send order")
            .json()
            .await
            .expect("Failed to read json");

        if order.success {
            match order_type {
                TradeSide::Buy => {
                    self.trade_active = true;
                }
                TradeSide::Sell => {
                    self.trade_active = false;
                }
            }
        }
    }

    pub async fn handle_message(&mut self, message: (Option<TradeSignal>, Option<f64>)) {
        if let Some(signal) = message.0 {
            match signal {
                TradeSignal::Buy => {}
                TradeSignal::Sell => {}
                TradeSignal::Hold => (),
            }
        }
    }

    // pub async fn get_order(&self, order_id: String) -> Option<CurrentOrder> {
    //     let timestamp = format!("{}", chrono::Utc::now().timestamp());

    //     let path = format!("{}/{}", "/api/v3/brokerage/orders/historical", order_id);

    //     let signature = http_sign(self.secret_key.as_bytes(), &timestamp, "GET", &path, "");

    //     let headers = self.create_headers(&timestamp, &signature);

    //     let url = format!("{}/{}", HISTORICAL_API_URL, order_id);

    //     let value = send_get_request::<CurrentOrderResponse>(&self.client, &url, headers).await;

    //     match value {
    //         Ok(v) => Some(v.order),
    //         Err(e) => {
    //             println!("ORDER VALUE Error: {}", e);
    //             None
    //         }
    //     }
    // }

    // pub async fn get_product(&self) -> Product {
    //     let timestamp = format!("{}", chrono::Utc::now().timestamp());

    //     let signature = http_sign(
    //         self.secret_key.as_bytes(),
    //         &timestamp,
    //         "GET",
    //         "/api/v3/brokerage/products/BTC-USD",
    //         "",
    //     );

    //     let headers = self.create_headers(&timestamp, &signature);

    //     let url = format!("{}/{}-{}", PRODUCT_API_URL, COIN_SYMBOL, BALANCE_CURRENCY);

    //     send_get_request::<Product>(&self.client, &url, headers)
    //         .await
    //         .expect("Failed to send message")
    // }

    pub fn is_trade_active(&self) -> bool {
        self.trade_active
    }
}
