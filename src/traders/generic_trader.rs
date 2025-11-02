use crate::account::Account;
use crate::data::BinanceKline;
use crate::indicators::BinanceIndicatorInstance;
use anyhow::Result;
use chrono::NaiveDateTime;
use log::debug;
use yata::core::Action;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum TradingFee {
    FixFee(f64),
    PercentageFee(f64),
}

#[derive(Clone, Copy)]
pub enum StakeSize {
    FixAmount(f64),
    FixPercentage(f64),
}

pub trait GenericTrader {
    fn stake_size(&self) -> StakeSize;
    fn trading_fee(&self) -> TradingFee;
    fn indicator(&mut self) -> &mut dyn BinanceIndicatorInstance;
    fn determine_trade(signals: &[Action]) -> Result<Action>;

    fn execute_buy(&self, timestamp: NaiveDateTime, price: f64, account: &mut Account) {
        let fund = account.available_fund;
        let stake = match self.stake_size() {
            StakeSize::FixAmount(amount) => {
                if amount <= fund {
                    amount
                } else {
                    0.
                }
            }
            StakeSize::FixPercentage(pct) => fund * pct,
        };
        let fee = match self.trading_fee() {
            TradingFee::FixFee(fee) => fee,
            TradingFee::PercentageFee(pct) => stake * pct / (1.0 - pct),
        };
        let quantity = (stake + fee) / price;

        if quantity > 0. {
            debug!("{}, B {:.08} @ ${:.08}, available_fund: {:.02}", timestamp, quantity, price, fund - stake);
            account.open(timestamp, quantity, price, fee);
        }
    }

    fn execute_sell(&self, timestamp: NaiveDateTime, price: f64, account: &mut Account) -> Result<()> {
        let current_position = account.position.quantity;
        let fee = match self.trading_fee() {
            TradingFee::FixFee(fee) => fee,
            TradingFee::PercentageFee(pct) => price * current_position * pct,
        };
        if current_position > 0. {
            debug!("{timestamp}, S {current_position:.08} @ $ {price:0.8}");
            account.close(timestamp, current_position, price, fee)?
        }

        Ok(())
    }

    fn next_trade_session(&mut self, account: &mut Account, kline: &BinanceKline) -> Result<()> {
        let timestamp = kline.end_time;
        let price = kline.close;

        let indicator = self.indicator().next_binance_kline(kline);
        let signals = indicator.signals();
        let trade = Self::determine_trade(signals)?;
        match trade {
            Action::Buy(_) => self.execute_buy(timestamp, price, account),
            Action::Sell(_) => self.execute_sell(timestamp, price, account)?,
            _ => debug!("Nothing to do"),
        };

        Ok(())
    }
}
