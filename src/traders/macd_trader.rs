use crate::data::BinanceKline;
use crate::indicators::BinanceIndicatorInstance;
use crate::traders::{GenericTrader, StakeSize, TradingFee};
use anyhow::{anyhow, Result};
use yata::core::{Action, IndicatorResult};
use yata::indicators::MACD;
use yata::prelude::dd::IndicatorInstanceDyn;
use yata::prelude::*;

use log::debug;

struct IndicatorInstanceWrapper(Box<dyn IndicatorInstanceDyn<BinanceKline>>);

impl BinanceIndicatorInstance for IndicatorInstanceWrapper {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult {
        self.0.next(candle)
    }
}

pub struct MACDTrader {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    indicator: IndicatorInstanceWrapper,
}

impl MACDTrader {
    pub fn new(kline_feed: &[BinanceKline], trading_fee: TradingFee, stake_size: StakeSize) -> Result<Self> {
        debug!("Creating a MACD Trader");
        let macd = MACD::default();
        let next_kline = kline_feed.first().ok_or(anyhow!("No klines in MACD feed"))?;
        let macd = macd.init(next_kline)?;
        Ok(Self { indicator: IndicatorInstanceWrapper(Box::new(macd)), trading_fee, stake_size })
    }
}

impl GenericTrader for MACDTrader {
    fn stake_size(&self) -> StakeSize {
        self.stake_size
    }

    fn trading_fee(&self) -> TradingFee {
        self.trading_fee
    }

    fn indicator(&mut self) -> &mut dyn BinanceIndicatorInstance {
        &mut self.indicator
    }

    fn determine_trade(signals: &[Action]) -> Result<Action> {
        debug!("Determine trades with MACD signal");
        let val = signals.get(1).ok_or(anyhow!("No MACD signal found"))?;
        Ok(*val)
    }
}
