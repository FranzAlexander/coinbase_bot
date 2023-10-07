use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::{
    coin::{Coin, CoinSymbol},
    model::{
        account::{AccountList, Product},
        event::{CoinbaseCandle, CoinbaseCandleEvent},
        order::OrderResponse,
        TradeSide,
    },
    util::{create_headers, get_api_string, send_get_request},
};

pub const WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";

const ACCOUNT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";
const PRODUCT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/products";
const ORDER_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders";
// const SUMMARY_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/transaction_summary";

const PRODUCT_REQUEST_PATH: &str = "/api/v3/brokerage/products";
const ACCOUNT_REQUEST_PATH: &str = "/api/v3/brokerage/accounts";
const ORDER_REQUEST_PATH: &str = "/api/v3/brokerage/orders";
// const SUMMARY_REQUEST_PATH: &str = "/api/v3/brokerage/transaction_summary";

#[derive(Debug)]
pub struct BotAccount {
    client: reqwest::Client,
    coins: Arc<Mutex<HashMap<CoinSymbol, Coin>>>,
    api_key: String,
    secret_key: String,
}

impl BotAccount {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");
        let secret_key = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

        let client = reqwest::Client::new();
        let coins = Arc::new(Mutex::new(HashMap::new()));

        BotAccount {
            client,
            coins,
            api_key,
            secret_key,
        }
    }

    pub async fn update_balances(&mut self) {
        let accounts = self.get_wallet().await;

        for account in accounts.accounts.into_iter() {
            let coin_symbol = self.map_currency_to_symbol(&account.available_balance.currency);
            if self.is_valid_coin(&coin_symbol) {
                let mut locked_coins = self.coins.lock().await;
                locked_coins.entry(coin_symbol).or_insert(Coin::new(
                    account.available_balance.value,
                    false,
                    0.0,
                    0.0,
                ));
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

    pub async fn get_product(&self, symbol: CoinSymbol) -> Product {
        let path = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_REQUEST_PATH);

        let headers = create_headers(self.secret_key.as_bytes(), &self.api_key, "GET", &path, "");
        let url = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_API_URL);

        send_get_request::<Product>(&self.client, &url, headers)
            .await
            .expect("Failed to get product")
    }

    pub async fn create_order(&mut self, order_type: TradeSide, symbol: CoinSymbol, atr: f64) {
        let price = self.get_product(symbol).await;

        let client_order_id = Uuid::new_v4().to_string();

        let amount = self.get_currency_amount(order_type, symbol).await;

        let (quote_size, base_size) = self.get_order_size(order_type, symbol, amount);

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
            let mut locked_coins = self.coins.lock().await;
            let coin = locked_coins.get_mut(&symbol).unwrap();
            match order_type {
                TradeSide::Buy => {
                    coin.update_coin(true, price.price - atr, price.price);
                }
                TradeSide::Sell => coin.update_coin(false, 0.0, 0.0),
            }
        }
    }

    pub fn get_order_size(
        &self,
        order_type: TradeSide,
        symbol: CoinSymbol,
        amount: f64,
    ) -> (Option<String>, Option<String>) {
        match order_type {
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
        }
    }

    async fn get_currency_amount(&self, order_type: TradeSide, symbol: CoinSymbol) -> f64 {
        if order_type == TradeSide::Buy {
            let mut count = 0;
            let locked_coins = self.coins.lock().await;
            for coin in locked_coins.iter() {
                if !coin.1.active_trade {
                    count += 1;
                }
            }
            locked_coins.get(&CoinSymbol::Usdc).unwrap().balance / count as f64
        } else {
            let locked_coins = self.coins.lock().await;
            locked_coins.get(&symbol).unwrap().balance
        }
    }

    fn map_currency_to_symbol(&self, currency: &str) -> CoinSymbol {
        match currency {
            "ADA" => CoinSymbol::Ada,
            "LINK" => CoinSymbol::Link,
            "USD" => CoinSymbol::Usd,
            "USDC" => CoinSymbol::Usdc,
            "XRP" => CoinSymbol::Xrp,
            "BTC" => CoinSymbol::Btc,
            "ETH" => CoinSymbol::Eth,
            // Add any other coin mappings here
            _ => CoinSymbol::Unknown,
        }
    }

    fn is_valid_coin(&self, coin_symbol: &CoinSymbol) -> bool {
        matches!(
            coin_symbol,
            // CoinSymbol::Ada // | CoinSymbol::Link
            |CoinSymbol::Usd| CoinSymbol::Usdc | CoinSymbol::Xrp // | CoinSymbol::Btc
                                                                 // | CoinSymbol::Eth
        )
    }

    #[inline]
    pub async fn coin_trade_active(&self, symbol: CoinSymbol) -> bool {
        self.coins.lock().await.get(&symbol).unwrap().active_trade
    }

    pub async fn update_coin_position(&mut self, symbol: CoinSymbol, high: f64, atr: f64) -> bool {
        let mut coins_guard = self.coins.lock().await;
        // Step 2: Borrow the value from the HashMap through the guard.
        if let Some(locked_coin) = coins_guard.get_mut(&symbol) {
            // Note the use of get_mut here

            if high > locked_coin.last_high {
                locked_coin.update_coin(true, high - atr, high);

                info!(
                    "HIGH: {}, LAST HIGH: {}, STOP LOSS:{}",
                    high, locked_coin.last_high, locked_coin.stop_loss
                );
            }

            if high <= locked_coin.stop_loss {
                info!(
                    "SELL, HIGH: {}, LAST HIGH: {}, STOP LOSS:{}",
                    high, locked_coin.last_high, locked_coin.stop_loss
                );
                return true;
                // self.create_order(TradeSide::Sell, symbol, atr, high).await;
            }
        }
        false
    }
}

pub fn get_product_candle(symbol: CoinSymbol, start: i64, end: i64) -> Vec<CoinbaseCandle> {
    let client = reqwest::blocking::Client::new();
    let api_string = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_REQUEST_PATH);
    let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");
    let secret_key = std::env::var("API_SECRET").expect("API_KEY not found in environment");

    let path = format!("{}/{}", api_string, "candles");
    let headers = create_headers(secret_key.as_bytes(), &api_key, "GET", &path, "");
    let url_string = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_API_URL);
    let url = format!(
        "{}/candles?start={}&end={}&granularity={}",
        url_string, start, end, "ONE_MINUTE"
    );

    let ans: CoinbaseCandleEvent = client
        .get(url)
        .headers(headers)
        .send()
        .unwrap()
        .json()
        .unwrap();

    ans.candles
}
