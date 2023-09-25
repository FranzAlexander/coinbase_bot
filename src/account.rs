use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

use uuid::Uuid;

use crate::{
    model::{
        account::{AccountList, Balance, Product},
        order::{CurrentOrder, CurrentOrderResponse, OrderResponse},
        TradeSide,
    },
    util::{create_headers, http_sign, send_get_request},
};

const ACCOUNT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";
const PRODUCT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/products";
const ORDER_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders";
const HISTORICAL_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders/historical";

const ORDER_REQUEST_PATH: &str = "/api/v3/brokerage/orders";

const STOP_LOSS_PERCENTAGE: f64 = 0.005;
const BALANCE_CURRENCY: &str = "USD";
const COIN_CURRENCY: &str = "USDC";
const COIN_SYMBOL: &str = "BTC";

#[derive(Debug)]
pub struct BotAccount {
    client: reqwest::Client,
    balances: Vec<Balance>,
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

        BotAccount {
            client,
            balances: Vec::new(),
            api_key,
            secret_key,
            trade_active: false,
        }
    }

    pub async fn update_balances(&mut self) {
        let accounts = self.get_wallet().await;

        println!("{:?}", accounts);

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
        let client_order_id = Uuid::new_v4().to_string();

        let currency = if order_type == TradeSide::Buy {
            COIN_CURRENCY
        } else {
            COIN_SYMBOL
        };

        let amount = self.get_balance_value_by_currency(currency);
        let (quote_size, base_size) = match order_type {
            TradeSide::Buy => (
                Some(format!("{:.2}", (amount * 100.0).floor() / 100.0)),
                None,
            ),
            TradeSide::Sell => (None, Some(format!("{:.8}", amount))),
        };

        let body = serde_json::json!({
            "client_order_id": client_order_id,
            "product_id":"BTC-USDC",
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

    pub async fn get_order(&self, order_id: String) -> Option<CurrentOrder> {
        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        let path = format!("{}/{}", "/api/v3/brokerage/orders/historical", order_id);

        let signature = http_sign(self.secret_key.as_bytes(), &timestamp, "GET", &path, "");

        let headers = self.create_headers(&timestamp, &signature);

        let url = format!("{}/{}", HISTORICAL_API_URL, order_id);

        let value = send_get_request::<CurrentOrderResponse>(&self.client, &url, headers).await;

        match value {
            Ok(v) => Some(v.order),
            Err(e) => {
                println!("ORDER VALUE Error: {}", e);
                None
            }
        }
    }

    pub async fn get_product(&self) -> Product {
        let timestamp = format!("{}", chrono::Utc::now().timestamp());

        let signature = http_sign(
            self.secret_key.as_bytes(),
            &timestamp,
            "GET",
            "/api/v3/brokerage/products/BTC-USD",
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

    pub fn is_trade_active(&self) -> bool {
        self.trade_active
    }
}
