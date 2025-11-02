use crate::data::BinanceKline;
use crate::indicators::BinanceIndicatorInstance;
use crate::indicators::Sma2Pair;
use crate::traders::{GenericTrader, StakeSize, TradingFee};
use anyhow::{anyhow, Result};
use yata::core::Action;
use yata::prelude::*;

use log::debug;

pub struct SMA2Trader {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    indicator: Box<dyn BinanceIndicatorInstance>,
}

impl SMA2Trader {
    pub fn new(kline_feed: &[BinanceKline], trading_fee: TradingFee) -> Result<Self> {
        debug!("Creating a SMA2 Trader");

        let sma_pair = Sma2Pair::new(1, 2);

        let next_kline = kline_feed.first().ok_or(anyhow!("No klines in SMA2 feed"))?;
        let sma = sma_pair.init(next_kline)?;
        Ok(Self { indicator: Box::new(sma), trading_fee, stake_size: StakeSize::FixAmount(100.0) })
    }
}

impl GenericTrader for SMA2Trader {
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
        debug!("Determine trades with SMA2 signal");
        let val = signals.first().ok_or(anyhow!("No SMA2 signal found"))?;
        Ok(*val)
    }
}
