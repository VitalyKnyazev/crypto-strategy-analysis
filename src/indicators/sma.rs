use crate::data::BinanceKline;
use crate::indicators::BinanceIndicatorInstance;
use anyhow::Result;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use yata::core::{Action, Error, IndicatorResult, OHLCV};
use yata::methods::SMA;
use yata::prelude::*;

#[derive(Debug, Clone)]
pub struct SmaPair {
    short_window: u8,
    long_window: u8,
}

impl SmaPair {
    pub fn new(short_window: u8, long_window: u8) -> Self {
        Self { short_window, long_window }
    }
}

#[derive(Debug, Clone)]
pub struct SMAInstance {
    cfg: SmaPair,
    sma1: SMA,
    sma2: SMA,
    last_timestamp: NaiveDateTime,
}

impl SMAInstance {
    fn new(cfg: SmaPair, first_value: f64) -> Result<Self, Error> {
        let short_window = cfg.short_window;
        let long_window = cfg.long_window;

        Ok(Self {
            last_timestamp: NaiveDate::from_ymd_opt(2000, 1, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).ok_or(Error::Other(String::from("Could not create last_timestamp")))?, // FIXME: a magic date before crypto happens
            cfg,
            sma1: SMA::new(short_window, &first_value)?,
            sma2: SMA::new(long_window, &first_value)?,
        })
    }
}

impl IndicatorConfig for SmaPair {
    type Instance = SMAInstance;

    const NAME: &'static str = "SMA";

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

impl IndicatorInstance for SMAInstance {
    type Config = SmaPair;

    fn config(&self) -> &Self::Config {
        &self.cfg
    }

    fn next<T: OHLCV>(&mut self, _candle: &T) -> IndicatorResult {
        IndicatorResult::new(&[], &[])
    }
}

impl BinanceIndicatorInstance for SMAInstance {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult {
        let current_time = candle.start_time;
        let current_month = current_time.month();
        let last_month = self.last_timestamp.month();
        self.last_timestamp = current_time;

        let short_ma = self.sma1.next(&candle.close);
        let long_ma = self.sma2.next(&candle.close);

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
