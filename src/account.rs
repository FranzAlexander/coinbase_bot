use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    coin::{Coin, CoinSymbol},
    model::{
        account::{AccountList, Product},
        event::{CoinbaseCandle, CoinbaseCandleEvent},
        fee::FeeData,
        order::OrderResponse,
        TradeSide,
    },
    trading_bot::TradeSignal,
    util::{create_headers, get_api_string, send_get_request},
};

pub const WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";

const ACCOUNT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/accounts";
const PRODUCT_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/products";
const ORDER_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/orders";
const SUMMARY_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage/transaction_summary";

const PRODUCT_REQUEST_PATH: &str = "/api/v3/brokerage/products";
const ACCOUNT_REQUEST_PATH: &str = "/api/v3/brokerage/accounts";
const ORDER_REQUEST_PATH: &str = "/api/v3/brokerage/orders";
const SUMMARY_REQUEST_PATH: &str = "/api/v3/brokerage/transaction_summary";

const XRP_HARD_STOP_LOSS: f64 = 0.0006;
const LINK_HARD_STOP_LOSS: f64 = 0.010;
const XRP_SOFT_STOP_LOSS: f64 = 0.0003;
const LINK_SOFT_STOP_LOSS: f64 = 0.004;

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
            let coin_symbol = match account.available_balance.currency.as_str() {
                "ADA" => CoinSymbol::Ada,
                "LINK" => CoinSymbol::Link,
                "USD" => CoinSymbol::Usd,
                "USDC" => CoinSymbol::Usdc,
                "XRP" => CoinSymbol::Xrp,
                "BTC" => CoinSymbol::Btc,
                "ETH" => CoinSymbol::Eth,
                // Add any other coin mappings here
                _ => CoinSymbol::Unknown,
            };
            match coin_symbol {
                CoinSymbol::Ada
                | CoinSymbol::Link
                | CoinSymbol::Usd
                | CoinSymbol::Usdc
                | CoinSymbol::Xrp
                | CoinSymbol::Btc
                | CoinSymbol::Eth => {
                    self.coins.entry(coin_symbol).or_insert(Coin::new(
                        account.available_balance.value,
                        false,
                        0.0,
                        0.0,
                    ));
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

    pub async fn create_order(
        &mut self,
        order_type: TradeSide,
        symbol: CoinSymbol,
        atr: f64,
        high: f64,
    ) {
        let price = self.get_product(symbol).await;
        let client_order_id = Uuid::new_v4().to_string();

        let amount = self.get_currency_amount(order_type.clone(), symbol);

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
            let coin = self.coins.get_mut(&symbol).unwrap();
            match order_type {
                TradeSide::Buy => {
                    let new_price = price.price - atr;

                    coin.update_coin(true, new_price, high);
                }
                TradeSide::Sell => coin.update_coin(false, 0.0, 0.0),
            }
        }
    }

    async fn get_product(&self, symbol: CoinSymbol) -> Product {
        let path = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_REQUEST_PATH);

        let headers = create_headers(self.secret_key.as_bytes(), &self.api_key, "GET", &path, "");
        let url = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_API_URL);

        send_get_request::<Product>(&self.client, &url, headers)
            .await
            .expect("Failed to get product")
    }

    #[inline]
    fn get_currency_amount(&self, order_type: TradeSide, symbol: CoinSymbol) -> f64 {
        if order_type == TradeSide::Buy {
            let mut count = 0;
            for coin in self.coins.iter() {
                if !coin.1.active_trade {
                    count += 1;
                }
            }
            self.coins.get(&CoinSymbol::Usdc).unwrap().balance / count as f64
        } else {
            self.coins.get(&symbol).unwrap().balance
        }
    }

    fn get_base_size(&self, symbol: CoinSymbol, order_type: TradeSide) -> String {
        if symbol == CoinSymbol::Xrp {
            if order_type == TradeSide::Buy {
                return format!("{:.4}", (50.0_f64 * 100.0).floor() / 100.0).to_string();
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

    #[inline]
    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    #[inline]
    pub fn coin_trade_active(&self, symbol: CoinSymbol) -> bool {
        self.coins.get(&symbol).unwrap().active_trade
    }

    pub async fn update_coin_position(&mut self, symbol: CoinSymbol, high: f64, atr: f64) -> bool {
        let coin = self.coins.get_mut(&symbol).unwrap();

        if high > coin.last_high {
            coin.last_high = high;
            coin.stop_loss = high - atr;
        }

        if high <= coin.stop_loss {
            return true;
            // self.create_order(TradeSide::Sell, symbol, atr, high).await;
        }
        false
    }

    pub fn get_coin(&self, symbol: CoinSymbol) {
        let coin = self.coins.get(&symbol);
        println!("{:?}", coin);
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
