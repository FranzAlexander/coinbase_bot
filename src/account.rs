use std::{collections::HashMap, sync::Arc};

use tracing::info;
use uuid::Uuid;

use crate::{
    coin::{Coin, CoinSymbol},
    model::{
        account::{Account, AccountList, AccountType, Product, SingleAccount},
        event::CandleHistory,
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

const XRP_SELL_PLACES: i32 = 6;
const XRP_BUY_PLACES: i32 = 4;
const BTC_SELL_PLACES: i32 = 8;
const BTC_BUY_PLACES: i32 = 2;

const USDC_BUY_PLACES: i32 = 2;

#[derive(Debug)]
pub struct BotAccount {
    client: reqwest::blocking::Client,
    api_key: String,
    secret_key: String,
    market_fee: f64,
    taker_fee: f64,
    div_num: usize, // The amount to which to divide total usdc by.
    can_trade: bool,
    symbol_id: Option<String>,
    usdc_id: Option<String>,
    stop_loss: f64,
    last_high: f64,
}

impl BotAccount {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");
        let secret_key = std::env::var("API_SECRET").expect("SECRET_KEY not found in environment");

        let client = reqwest::blocking::Client::new();

        BotAccount {
            client,
            api_key,
            secret_key,
            market_fee: 0.0,
            taker_fee: 0.0,
            div_num: 0,
            can_trade: true,
            symbol_id: None,
            usdc_id: None,
            stop_loss: 0.0,
            last_high: 0.0,
        }
    }

    pub fn update_balances(&mut self, symbol: CoinSymbol) {
        // Get coins in coinbase wallet.
        let accounts = self.get_wallet();

        // The current fees charged by coinbase.
        let fees = self.get_transaction_summary();
        self.market_fee = fees.fee_tier.maker_fee_rate;
        self.taker_fee = fees.fee_tier.taker_fee_rate;

        // Loop through each coin account in wallet.
        for account in accounts.accounts.iter() {
            // Map coinbase coin string to enum type.
            let coin_symbol = self.map_currency_to_symbol(&account.available_balance.currency);

            // Check to see if the coin is part of a list of used coins by the bot.
            if self.is_valid_coin(&coin_symbol) && account.account_type == AccountType::Crypto {
                if coin_symbol == symbol && self.symbol_id.is_none() {
                    self.symbol_id = Some(account.uuid.clone());
                }

                if coin_symbol == CoinSymbol::Usdc && self.usdc_id.is_none() {
                    self.usdc_id = Some(account.uuid.clone());
                }

                // Checks to see if current amount held is 0.
                let value = self.check_coin_amount(coin_symbol, account.available_balance.value);

                // Check to see if there is any coin amount held by account.
                if value == 0.0 {
                    self.div_num += 1;
                } else {
                    if self.div_num > 1 {
                        self.div_num -= 1;
                    }
                }
            }
        }
    }

    fn truncate_to_decimal_places(&self, num: f64, places: i32) -> f64 {
        let multipler = 10_f64.powi(places);
        (num * multipler).floor() / multipler
    }

    fn check_coin_amount(&self, coin_symbol: CoinSymbol, value: f64) -> f64 {
        let order_type = if coin_symbol == CoinSymbol::Usdc {
            TradeSide::Buy
        } else {
            TradeSide::Sell
        };

        let coin_places = self.get_coin_places(&coin_symbol, order_type);

        self.truncate_to_decimal_places(value, coin_places)
    }

    fn get_wallet(&self) -> AccountList {
        let headers = create_headers(
            self.secret_key.as_bytes(),
            &self.api_key,
            "GET",
            ACCOUNT_REQUEST_PATH,
            "",
        );

        send_get_request::<AccountList>(&self.client, ACCOUNT_API_URL, headers)
            .expect("Failed to send get request!")
    }

    pub fn get_product(&self, symbol: CoinSymbol) -> Product {
        let path = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_REQUEST_PATH);

        let headers = create_headers(self.secret_key.as_bytes(), &self.api_key, "GET", &path, "");
        let url = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_API_URL);

        send_get_request::<Product>(&self.client, &url, headers).expect("Failed to get product")
    }

    fn get_transaction_summary(&self) -> FeeData {
        let headers = create_headers(
            self.secret_key.as_bytes(),
            &self.api_key,
            "GET",
            SUMMARY_REQUEST_PATH,
            "",
        );

        send_get_request::<FeeData>(&self.client, SUMMARY_API_URL, headers)
            .expect("Failed to send summary get request!")
    }

    pub fn get_account(&self, order_type: TradeSide) -> SingleAccount {
        let id = if order_type == TradeSide::Buy {
            self.usdc_id.clone().unwrap()
        } else {
            self.symbol_id.clone().unwrap()
        };

        let path = format!("{}/{}", ACCOUNT_REQUEST_PATH, id);
        let headers = create_headers(self.secret_key.as_bytes(), &self.api_key, "GET", &path, "");
        let url_string = format!("{}/{}", ACCOUNT_API_URL, id);

        send_get_request::<SingleAccount>(&self.client, &url_string, headers).unwrap()
    }

    pub fn create_order(&mut self, order_type: TradeSide, symbol: CoinSymbol, atr: f64) {
        let client_order_id = Uuid::new_v4().to_string();

        let amount = self.get_currency_amount(order_type, symbol);

        println!("Amount: {}", amount);

        let mut quote_size: Option<String>;
        let mut base_size: Option<String>;

        match order_type {
            TradeSide::Buy => {
                quote_size = Some(amount.to_string());
                base_size = None;
            }
            TradeSide::Sell => {
                base_size = Some(amount.to_string());
                quote_size = None;
            }
        };

        let product_id = format!(
            "{}-{}",
            String::from(symbol),
            String::from(CoinSymbol::Usdc)
        );

        let price = self.get_product(symbol);

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
            .expect("Failed to send order")
            .json()
            .expect("Failed to read json");

        if order.success {
            match order_type {
                TradeSide::Buy => {
                    self.can_trade = false;
                    self.stop_loss = price.price - atr;
                    self.last_high = price.price;
                }
                TradeSide::Sell => {
                    self.can_trade = true;
                    self.stop_loss = 0.0;
                    self.last_high = 0.0;
                }
            }
        }
    }

    fn get_currency_amount(&self, order_type: TradeSide, symbol: CoinSymbol) -> f64 {
        let value = self.get_account(order_type).account.available_balance.value;

        let new_value = if order_type == TradeSide::Buy {
            value / self.div_num as f64
        } else {
            value
        };
        let places = self.get_coin_places(&symbol, order_type);
        self.truncate_to_decimal_places(new_value, places)
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
            // CoinSymbol::Ada
            CoinSymbol::Usd | CoinSymbol::Usdc | CoinSymbol::Xrp | CoinSymbol::Btc // | CoinSymbol::Eth
        )
    }

    fn get_coin_places(&self, coin_symbol: &CoinSymbol, order_type: TradeSide) -> i32 {
        match order_type {
            TradeSide::Buy => match coin_symbol {
                CoinSymbol::Usdc => USDC_BUY_PLACES,
                CoinSymbol::Xrp => XRP_BUY_PLACES,
                CoinSymbol::Btc => BTC_BUY_PLACES,
                _ => 0,
            },
            TradeSide::Sell => match coin_symbol {
                CoinSymbol::Xrp => XRP_SELL_PLACES,
                CoinSymbol::Btc => BTC_SELL_PLACES,
                _ => 0,
            },
        }
    }

    pub fn can_trade(&self) -> bool {
        self.can_trade
    }

    pub fn update_coin_position(&mut self, high: f64, atr: f64) -> bool {
        if high > self.last_high {
            self.stop_loss = high - atr;
            self.last_high = high;

            println!(
                "HIGH: {}, LAST HIGH: {}, STOP LOSS:{}",
                high, self.last_high, self.stop_loss
            );
        }

        if high <= self.stop_loss {
            println!(
                "SELL, HIGH: {}, LAST HIGH: {}, STOP LOSS:{}",
                high, self.last_high, self.stop_loss
            );
            return true;
        }
        false
    }
}

pub fn get_product_candle(symbol: CoinSymbol, start: i64, end: i64) -> CandleHistory {
    let client = reqwest::blocking::Client::new();
    let api_string = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_REQUEST_PATH);
    let api_key = std::env::var("API_KEY").expect("API_KEY not found in environment");
    let secret_key = std::env::var("API_SECRET").expect("API_KEY not found in environment");

    let path = format!("{}/{}", api_string, "candles");
    let headers = create_headers(secret_key.as_bytes(), &api_key, "GET", &path, "");
    let url_string = get_api_string(symbol, CoinSymbol::Usdc, PRODUCT_API_URL);
    let url = format!(
        "{}/candles?start={}&end={}&granularity={}",
        url_string, start, end, "FIVE_MINUTE"
    );

    let ans: CandleHistory = client
        .get(url)
        .headers(headers)
        .send()
        .unwrap()
        .json()
        .unwrap();

    ans
}
