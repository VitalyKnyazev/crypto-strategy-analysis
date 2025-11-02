use crate::data::BinanceKline;
use crate::indicators::BinanceIndicatorInstance;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use yata::core::{Action, Error, IndicatorResult, OHLCV};
use yata::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct Sma2 {
    window_size: usize,
    prices: Vec<f64>,
}

impl Sma2 {
    pub fn new(window_size: usize, value: f64) -> Self {
        Self { window_size, prices: vec![value] }
    }

    fn update_price(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);
        self.moving_average()
    }

    fn moving_average(&self) -> Option<f64> {
        if self.prices.len() < self.window_size {
            return None;
        }
        let slice = &self.prices[self.prices.len() - self.window_size..];
        Some(slice.iter().sum::<f64>() / self.window_size as f64)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Sma2Pair {
    short_window: usize,
    long_window: usize,
}

impl Sma2Pair {
    pub fn new(short_window: usize, long_window: usize) -> Self {
        Self { short_window, long_window }
    }
}

#[derive(Debug, Clone)]
pub struct SMA2Instance {
    cfg: Sma2Pair,
    sma1: Sma2,
    sma2: Sma2,
    last_timestamp: NaiveDateTime,
}

impl SMA2Instance {
    fn new(cfg: Sma2Pair, first_value: f64) -> Result<Self, Error> {
        let short_window = cfg.short_window;
        let long_window = cfg.long_window;

        Ok(Self {
            last_timestamp: NaiveDate::from_ymd_opt(2000, 1, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).ok_or(Error::Other(String::from("Could not create last_timestamp")))?, // FIXME: a magic date before crypto happens
            cfg,
            sma1: Sma2::new(short_window, first_value),
            sma2: Sma2::new(long_window, first_value),
        })
    }
}

impl IndicatorConfig for Sma2Pair {
    type Instance = SMA2Instance;

    const NAME: &'static str = "SMA2";

    fn init<T: OHLCV>(self, candle: &T) -> Result<Self::Instance, Error> {
        Self::Instance::new(self, candle.close())
    }
    fn validate(&self) -> bool {
        true
    }
    fn set(&mut self, _name: &str, _value: String) -> Result<(), Error> {
        Ok(())
    }
    fn size(&self) -> (u8, u8) {
        (0, 1)
    }
}

impl IndicatorInstance for SMA2Instance {
    type Config = Sma2Pair;

    fn config(&self) -> &Self::Config {
        &self.cfg
    }

    fn next<T: OHLCV>(&mut self, _candle: &T) -> IndicatorResult {
        IndicatorResult::new(&[], &[])
    }
}

impl BinanceIndicatorInstance for SMA2Instance {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult {
        let current_time = candle.start_time;
        let current_month = current_time.month();
        let last_month = self.last_timestamp.month();
        self.last_timestamp = current_time;

        let short_ma = self.sma1.update_price(candle.close);
        let long_ma = self.sma2.update_price(candle.close);

        let action = if current_month == last_month {
            Action::None
        } else if short_ma > long_ma {
            Action::Buy(1)
        } else if short_ma < long_ma {
            Action::Sell(1)
        } else {
            Action::None
        };

        IndicatorResult::new(&[], &[action])
    }
}
