use std::{collections::HashMap, time::SystemTime};

use chrono::{Duration, Utc};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

use serde_json::Value;
use tracing::info;
use uuid::Uuid;

use crate::{
    coin::{Coin, CoinSymbol},
    model::{
        account::{AccountList, Balance, Product},
        fee::FeeData,
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
const SUMMARY_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/transaction_summary";

const PRODUCT_REQUEST_PATH: &str = "/api/v3/brokerage/products";
const ACCOUNT_REQUEST_PATH: &str = "/api/v3/brokerage/accounts";
const ORDER_REQUEST_PATH: &str = "/api/v3/brokerage/orders";
const SUMMARY_REQUEST_PATH: &str = "/api/v3/brokerage/transaction_summary";

#[derive(Debug)]
pub struct BotAccount {
    client: reqwest::Client,
    coins: HashMap<CoinSymbol, Coin>,
    api_key: String,
    secret_key: String,
    fees: Option<FeeData>,
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
            fees: None,
        }
    }

    pub async fn update_balances(&mut self) {
        let accounts = self.get_wallet().await;
        let fee_data = self.get_transaction_summary().await;
        self.fees = Some(fee_data);

        for account in accounts.accounts.into_iter() {
            match account.currency {
                CoinSymbol::Ada
                | CoinSymbol::Link
                | CoinSymbol::Usd
                | CoinSymbol::Usdc
                | CoinSymbol::Xrp
                | CoinSymbol::Btc
                | CoinSymbol::Eth => {
                    self.coins
                        .insert(account.currency, Coin::new(account.available_balance.value));
                }
                CoinSymbol::Unknown => (),
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

    async fn get_transaction_summary(&self) -> FeeData {
        let headers = create_headers(
            self.secret_key.as_bytes(),
            &self.api_key,
            "GET",
            SUMMARY_REQUEST_PATH,
            "",
        );

        send_get_request::<FeeData>(&self.client, SUMMARY_API_URL, headers)
            .await
            .expect("Failed to send get request!")
    }

    pub async fn create_order(&mut self, order_type: TradeSide, symbol: CoinSymbol) {
        let client_order_id = Uuid::new_v4().to_string();

        let amount = self.get_currency_amount(order_type.clone(), &symbol);

        let (quote_size, base_size) = match order_type {
            TradeSide::Buy => (
                if symbol == CoinSymbol::Xrp {
                    Some(format!("{:.4}", (amount * 100.0).floor() / 100.0))
                } else {
                    Some(format!("{:.3}", (amount * 100.0).floor() / 100.0))
                },
                None,
            ),
            TradeSide::Sell => (
                None,
                if symbol == CoinSymbol::Xrp {
                    Some(format!("{:.6}", (amount * 100.0).floor() / 100.0))
                } else {
                    Some(format!("{:.3}", (amount * 100.0).floor() / 100.0))
                },
            ),
        };

        let product_id = format!(
            "{}-{}",
            String::from(symbol),
            String::from(CoinSymbol::Usdc)
        );

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
                TradeSide::Buy => {}
                TradeSide::Sell => {}
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

    async fn get_product(&self, symbol: CoinSymbol) -> Product {
        let path = self.get_api_string(symbol.clone(), CoinSymbol::Usdc, PRODUCT_REQUEST_PATH);

        let headers = create_headers(&self.secret_key.as_bytes(), &self.api_key, "GET", &path, "");
        let url = self.get_api_string(symbol.clone(), CoinSymbol::Usdc, PRODUCT_API_URL);

        send_get_request::<Product>(&self.client, &url, headers)
            .await
            .expect("Failed to get product")
    }

    #[inline]
    fn get_api_string(&self, symbol: CoinSymbol, currency: CoinSymbol, endpoint: &str) -> String {
        format!(
            "{}/{}-{}",
            endpoint,
            String::from(symbol),
            String::from(currency)
        )
    }

    #[inline]
    fn get_currency_amount(&self, order_type: TradeSide, symbol: &CoinSymbol) -> f64 {
        if order_type == TradeSide::Buy {
            self.coins.get(&CoinSymbol::Usdc).unwrap().balance
        } else {
            self.coins.get(symbol).unwrap().balance
        }
    }

    fn get_base_size(&self, symbol: CoinSymbol, order_type: TradeSide) -> String {
        if symbol == CoinSymbol::Xrp {
            if order_type == TradeSide::Buy {
                return format!("{:.4}", (5.0_f64 * 100.0).floor() / 100.0).to_string();
                // return format!(
                //     "{:.4}",
                //     (self.get_currency_amount(order_type, &symbol) * 100.0).floor() / 100
                // )
                // .to_string();
            } else {
                return "".to_string();
            }
        }
        "".to_string()
    }
}
