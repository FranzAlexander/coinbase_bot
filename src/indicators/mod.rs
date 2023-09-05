use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};

use rayon::{ThreadBuilder, ThreadPoolBuilder};

use crate::model::OneMinuteCandle;

use self::{ema::Ema, macd::Macd};

pub mod ema;
pub mod macd;

pub enum IndicatorType {
    Candlestick(OneMinuteCandle),
}

pub struct BotIndicator {
    tx: Sender<IndicatorType>,
    short_ema: Arc<Mutex<Ema>>,
    long_ema: Arc<Mutex<Ema>>,
    macd: Arc<Mutex<Macd>>,
}

impl BotIndicator {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let short_ema = Arc::new(Mutex::new(Ema::new(12)));
        let long_ema = Arc::new(Mutex::new(Ema::new(26)));
        let macd = Arc::new(Mutex::new(Macd::new(12, 26, 9)));

        let pool = ThreadPoolBuilder::new().num_threads(3).build().unwrap();

        let local_short_ema = short_ema.clone();
        let local_long_ema = long_ema.clone();
        let local_macd = macd.clone();

        pool.spawn(move || {
            Self::process_calculation(rx, local_short_ema, local_long_ema, local_macd)
        });

        BotIndicator {
            tx,
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
                }
            }
        }
    }

    pub fn send_to_processing(&self, data: IndicatorType) {
        self.tx.send(data).expect("Failed to send to channel");
    }
}
