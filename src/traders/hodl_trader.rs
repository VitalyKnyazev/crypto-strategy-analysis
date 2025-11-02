use crate::data::BinanceKline;
use crate::indicators::BinanceIndicatorInstance;
use crate::indicators::Hodl;
use crate::traders::{GenericTrader, StakeSize, TradingFee};
use anyhow::{anyhow, Result};
use yata::core::Action;
use yata::prelude::*;

use log::debug;

pub struct HODLTrader {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    indicator: Box<dyn BinanceIndicatorInstance>,
}

impl HODLTrader {
    pub fn new(kline_feed: &[BinanceKline], trading_fee: TradingFee) -> Result<Self> {
        debug!("Creating a HODL Trader");
        let hodl = Hodl;
        let next_kline = kline_feed.first().ok_or(anyhow!("No klines in HODL feed"))?;
        let hodl = hodl.init(next_kline)?;
        Ok(Self { indicator: Box::new(hodl), trading_fee, stake_size: StakeSize::FixPercentage(1.) })
    }
}

impl GenericTrader for HODLTrader {
    fn stake_size(&self) -> StakeSize {
        self.stake_size
    }

    fn trading_fee(&self) -> TradingFee {
        self.trading_fee
    }

    fn indicator(&mut self) -> &mut dyn BinanceIndicatorInstance {
        self.indicator.as_mut()
    }

    fn determine_trade(signals: &[Action]) -> Result<Action> {
        debug!("Determine trades with hodl signal");
        let val = signals.first().ok_or(anyhow!("No hodl signal found"))?;
        Ok(*val)
    }
}
