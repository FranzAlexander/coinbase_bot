use super::ema::Ema;

#[derive(Debug)]
pub struct Macd {
    short_ema: Ema,
    long_ema: Ema,
    signal: Ema,
}
