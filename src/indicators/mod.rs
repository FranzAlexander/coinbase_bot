use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};

use rayon::{ThreadBuilder, ThreadPoolBuilder};
use rust_decimal_macros::dec;

use crate::model::OneMinuteCandle;

use self::{ema::Ema, macd::Macd, rsi::Rsi};

pub mod ema;
pub mod macd;
pub mod rsi;

pub enum IndicatorType {
    Candlestick(OneMinuteCandle),
}

pub struct BotIndicator {
    tx: Sender<IndicatorType>,
    short_ema: Arc<Mutex<Ema>>,
    long_ema: Arc<Mutex<Ema>>,
    macd: Arc<Mutex<Macd>>,
    rsi: Arc<Mutex<Rsi>>,
}

impl BotIndicator {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let short_ema = Arc::new(Mutex::new(Ema::new(12)));
        let long_ema = Arc::new(Mutex::new(Ema::new(26)));
        let macd = Arc::new(Mutex::new(Macd::new(12, 26, 9)));

        let rsi = Arc::new(Mutex::new(Rsi::new(14)));

        let pool = ThreadPoolBuilder::new().num_threads(3).build().unwrap();

        let local_short_ema = short_ema.clone();
        let local_long_ema = long_ema.clone();
        let local_macd = macd.clone();
        let local_rsi = rsi.clone();

        pool.spawn(move || {
            Self::process_calculation(rx, local_short_ema, local_long_ema, local_macd, local_rsi)
        });

        BotIndicator {
            tx,
            short_ema,
            long_ema,
            macd,
            rsi,
        }
    }

    fn process_calculation(
        rx: Receiver<IndicatorType>,
        short_ema: Arc<Mutex<Ema>>,
        long_ema: Arc<Mutex<Ema>>,
        macd: Arc<Mutex<Macd>>,
        rsi: Arc<Mutex<Rsi>>,
    ) {
        while let Ok(indactor_type) = rx.recv() {
            match indactor_type {
                IndicatorType::Candlestick(candle) => {
                    {
                        let mut local_short_ema = short_ema.lock().unwrap();
                        local_short_ema.update(candle.close.unwrap());
                        println!("{:?}", local_short_ema.get_current_ema());
                    }
                    {
                        let mut local_long_ema = long_ema.lock().unwrap();
                        local_long_ema.update(candle.close.unwrap());
                        println!("{:?}", local_long_ema.get_current_ema());
                    }
                    {
                        let mut local_macd = macd.lock().unwrap();
                        local_macd.update(candle.close.unwrap());
                        println!("{:?}", local_macd.get_signal_n_macd());
                    }
                    {
                        let mut local_rsi = rsi.lock().unwrap();
                        local_rsi.update(candle.close.unwrap());
                        println!("{:?}", local_rsi.get_rsi());
                    }
                }
            }
        }
    }

    pub fn send_to_processing(&self, data: IndicatorType) {
        self.tx.send(data).expect("Failed to send to channel");
    }

    // pub fn check_signal(&self) {
    //     let local_short_ema = self.short_ema.lock().unwrap();
    //     let local_long_ema = self.long_ema.lock().unwrap();
    //     let local_macd = self.macd.lock().unwrap();
    //     let local_rsi = self.rsi.lock().unwrap();

    //     let mut short_ema = dec!(0.0);
    //     let mut long_ema = dec!(0.0);
    //     let mut macd = (dec!(0.0), dec!(0.0));
    //     let rsi = local_rsi.get_rsi();

    //     match local_short_ema.get_current_ema() {
    //         Some(ema) => short_ema = ema,
    //         None => return,
    //     }

    //     match local_long_ema.get_current_ema() {
    //         Some(ema) => long_ema = ema,
    //         None => return,
    //     }

    //     match local_macd.get_signal_n_macd() {
    //         (Some(signal), Some(macd_v)) => {
    //             macd.0 = signal;
    //             macd.1 = macd_v
    //         }
    //         (None, None) => return,
    //         _ => {
    //             return;
    //         }
    //     }

    //     if short_ema > long_ema && macd.0 < macd.1 && rsi < dec!(30.0) {
    //         println!("BUY");
    //     } else {
    //         println!("SELL");
    //     }
    // }

    pub fn check_signal(&self) {
        // Lock and directly try to retrieve values using pattern matching
        if let (Some(short_ema), Some(long_ema), Some(macd_signal), Some(macd_v), rsi) = (
            self.short_ema.lock().unwrap().get_current_ema(),
            self.long_ema.lock().unwrap().get_current_ema(),
            self.macd.lock().unwrap().get_signal_n_macd().0,
            self.macd.lock().unwrap().get_signal_n_macd().1,
            self.rsi.lock().unwrap().get_rsi(),
        ) {
            // Check conditions and print BUY/SELL
            if short_ema > long_ema && macd_signal < macd_v && rsi < dec!(30.0) {
                println!("BUY");
            } else {
                println!("SELL");
            }
        } else {
            // In case of any None value from the above indicators, we'll just exit the function.
            return;
        }
    }
}
