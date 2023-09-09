use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
};

use rust_decimal_macros::dec;

use crate::model::OneMinuteCandle;

use self::{adx::Adx, ema::Ema, macd::Macd, rsi::Rsi};

mod adx;
pub mod ema;
pub mod macd;
pub mod rsi;

pub enum IndicatorType {
    Candlestick(OneMinuteCandle),
    Shutdown(bool),
}

pub struct BotIndicator {
    short_ema: Ema,
    long_ema: Ema,
    macd: Macd,
}

impl BotIndicator {
    pub fn new() -> Self {
        let short_ema = Ema::new(12);
        let long_ema = Ema::new(26);
        let macd = Macd::new(9);

        BotIndicator {
            short_ema,
            long_ema,
            macd,
        }
    }

    fn process_calculation(
        rx: Receiver<IndicatorType>,
        short_ema: Arc<Mutex<Ema>>,
        long_ema: Arc<Mutex<Ema>>,
        macd: Arc<Mutex<Macd>>,
        rsi: Arc<Mutex<Rsi>>,
        adx: Arc<Mutex<Adx>>,
    ) {
        while let Ok(indactor_type) = rx.recv() {
            match indactor_type {
                IndicatorType::Candlestick(candle) => {
                    // {
                    //     let mut local_short_ema = short_ema.lock().unwrap();
                    //     local_short_ema.update(candle.close.unwrap());
                    //     println!("{:?}", local_short_ema.get_current_ema());
                    // }
                    // {
                    //     let mut local_long_ema = long_ema.lock().unwrap();
                    //     local_long_ema.update(candle.close.unwrap());
                    //     println!("{:?}", local_long_ema.get_current_ema());
                    // }
                    // {
                    //     let mut local_macd = macd.lock().unwrap();
                    //     local_macd.update(candle.close.unwrap());
                    //     println!("{:?}", local_macd.get_signal_n_macd());
                    // }
                    // {
                    //     let mut local_rsi = rsi.lock().unwrap();
                    //     local_rsi.update(candle.close.unwrap());
                    //     println!("{:?}", local_rsi.get_rsi());
                    // }
                    // {
                    //     let mut local_adx = adx.lock().unwrap();
                    //     local_adx.update(
                    //         candle.high.unwrap(),
                    //         candle.low.unwrap(),
                    //         candle.close.unwrap(),
                    //     );
                    //     println!("{:?}", local_adx.get_adx());
                    // }
                }
                IndicatorType::Shutdown(shutdown) => {
                    if shutdown {
                        break;
                    }
                }
            }
        }
    }

    pub fn send_to_processing(&self, data: IndicatorType) {
        // self.tx.send(data).expect("Failed to send to channel");
    }

    pub fn check_signal(&self) {
        // let short_ema = self.short_ema.lock().unwrap().get_current_ema();
        // let long_ema = self.long_ema.lock().unwrap().get_current_ema();

        // let macd_values = self.macd.lock().unwrap().get_signal_n_macd();
        // let rsi = self.rsi.lock().unwrap().get_rsi();

        // if let (
        //     Some(short_ema_value),
        //     Some(long_ema_value),
        //     Some(macd_signal),
        //     Some(macd_v),
        //     rsi_value,
        // ) = (short_ema, long_ema, macd_values.0, macd_values.1, rsi)
        // {
        //     if short_ema_value > long_ema_value && macd_signal < macd_v && rsi_value < dec!(30.0) {
        //         println!("BUY");
        //     } else {
        //         println!("SELL");
        //     }
        // }
    }
}
