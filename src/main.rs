mod account;
mod data;
mod indicators;
mod traders;

use account::{Account, Position};
use chrono::{Duration, NaiveDate, Utc};
use data::{get_kline_data, BinanceKline};
use traders::{DCATrader, GenericTrader, HODLTrader, MACDTrader, SMA2Trader, SMATrader, StakeSize, TradingFee};

use env_logger::Env;
use log::info;

use anyhow::{anyhow, Result};

use std::sync::Arc;
use std::thread;

use my_macros::log_duration;

#[log_duration]
async fn download_kline() -> Result<Vec<BinanceKline>> {
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).ok_or(anyhow!("Invalid start date"))?;
    let end_date = Utc::now().naive_utc() - Duration::days(1);
    let end_date = end_date.date();
    let symbol = "ETHUSDT";
    let interval = "1h";
    info!("Download data from binance for [{symbol}/{interval}] from [{start_date}] to [{end_date}]");
    let klines = get_kline_data(symbol, interval, start_date, end_date).await?;
    info!("Downloaded [{}] klines", klines.len());
    Ok(klines)
}

fn initialise_account(klines: &[BinanceKline], name: &str) -> Result<Account> {
    info!("Setting up account {name}");
    let first_kline = klines.first().ok_or(anyhow!("No klines fetched"))?;
    let start_time = first_kline.start_time;
    let start_fund = 1000.0;
    let start_position = Position { quantity: 0.0, cost: 0.0 };
    let account = Account::new(start_fund, start_position, start_time);
    Ok(account)
}

fn initialise_macd_trader(klines: &[BinanceKline]) -> Result<MACDTrader> {
    info!("Setting up MACD trader");
    let stake_size = StakeSize::FixPercentage(1.);
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = MACDTrader::new(klines, trading_fee, stake_size)?;
    Ok(trader)
}

fn initialise_hodl_trader(klines: &[BinanceKline]) -> Result<HODLTrader> {
    info!("Setting up HODL trader");
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = HODLTrader::new(klines, trading_fee)?;
    Ok(trader)
}

fn initialise_dca_trader(klines: &[BinanceKline]) -> Result<DCATrader> {
    info!("Setting up DCA trader");
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = DCATrader::new(klines, trading_fee)?;
    Ok(trader)
}

fn initialise_sma_trader(klines: &[BinanceKline]) -> Result<SMATrader> {
    info!("Setting up SMA trader");
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = SMATrader::new(klines, trading_fee)?;
    Ok(trader)
}

fn initialise_sma2_trader(klines: &[BinanceKline]) -> Result<SMA2Trader> {
    info!("Setting up SMA2 trader");
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = SMA2Trader::new(klines, trading_fee)?;
    Ok(trader)
}

fn loop_kline<T>(trader: &mut T, account: &mut Account, name: &str, klines: &[BinanceKline]) -> Result<()>
where
    T: GenericTrader,
{
    info!("Running backtest {name}");
    for kline in klines {
        trader.next_trade_session(account, kline)?;
        account.mark_to_market(kline.end_time, kline.close)?;
    }

    Ok(())
}

#[log_duration]
async fn backtest_macd(klines: Arc<Vec<BinanceKline>>, name: &str) -> Result<Account> {
    let mut account = initialise_account(&klines, name)?;
    let mut trader = initialise_macd_trader(&klines)?;
    info!("MACD thread id: {:?}", thread::current().id());
    loop_kline(&mut trader, &mut account, name, &klines)?;
    Ok(account)
}

#[log_duration]
async fn backtest_hodl(klines: Arc<Vec<BinanceKline>>, name: &str) -> Result<Account> {
    let mut account = initialise_account(&klines, name)?;
    let mut trader = initialise_hodl_trader(&klines)?;
    info!("HODL thread id: {:?}", thread::current().id());
    loop_kline(&mut trader, &mut account, name, &klines)?;
    Ok(account)
}

#[log_duration]
async fn backtest_dca(klines: Arc<Vec<BinanceKline>>, name: &str) -> Result<Account> {
    let mut account = initialise_account(&klines, name)?;
    let mut trader = initialise_dca_trader(&klines)?;
    info!("DCA thread id: {:?}", thread::current().id());
    loop_kline(&mut trader, &mut account, name, &klines)?;
    Ok(account)
}

#[log_duration]
async fn backtest_sma(klines: Arc<Vec<BinanceKline>>, name: &str) -> Result<Account> {
    let mut account = initialise_account(&klines, name)?;
    let mut trader = initialise_sma_trader(&klines)?;
    info!("SMA thread id: {:?}", thread::current().id());
    loop_kline(&mut trader, &mut account, name, &klines)?;
    Ok(account)
}

#[log_duration]
async fn backtest_sma2(klines: Arc<Vec<BinanceKline>>, name: &str) -> Result<Account> {
    let mut account = initialise_account(&klines, name)?;
    let mut trader = initialise_sma2_trader(&klines)?;
    info!("SMA2 thread id: {:?}", thread::current().id());
    loop_kline(&mut trader, &mut account, name, &klines)?;
    Ok(account)
}

async fn backtest(klines: Vec<BinanceKline>) -> Result<(Result<Account>, Result<Account>, Result<Account>, Result<Account>, Result<Account>)> {
    info!("Main thread id: {:?}", thread::current().id());

    let klines = Arc::new(klines);

    let macd_account_handle = tokio::spawn(backtest_macd(Arc::clone(&klines), "MACD"));
    let hodl_account_handle = tokio::spawn(backtest_hodl(Arc::clone(&klines), "HODL"));
    let dca_account_handle = tokio::spawn(backtest_dca(Arc::clone(&klines), "DCA"));
    let sma_account_handle = tokio::spawn(backtest_sma(Arc::clone(&klines), "SMA"));
    let sma2_account_handle = tokio::spawn(backtest_sma2(Arc::clone(&klines), "SMA2"));

    let (macd_account, hodl_account, dca_account, sma_account, sma2_account) = tokio::join!(macd_account_handle, hodl_account_handle, dca_account_handle, sma_account_handle, sma2_account_handle);

    Ok((macd_account?, hodl_account?, dca_account?, sma_account?, sma2_account?))
}

#[tokio::main]
pub async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let klines = download_kline().await?;

    let result = backtest(klines);
    let (macd_account, hodl_account, dca_account, sma_account, sma2_account) = result.await?;

    info!("MACD: {:?}", macd_account?.profit_and_loss_history.last().ok_or(anyhow!("No pnl history for MACD"))?);
    info!("HODL: {:?}", hodl_account?.profit_and_loss_history.last().ok_or(anyhow!("No pnl history for HODL"))?);
    info!("DCA : {:?}", dca_account?.profit_and_loss_history.last().ok_or(anyhow!("No pnl history for DCA"))?);
    info!("SMA : {:?}", sma_account?.profit_and_loss_history.last().ok_or(anyhow!("No pnl history for SMA"))?);
    info!("SMA2 : {:?}", sma2_account?.profit_and_loss_history.last().ok_or(anyhow!("No pnl history for SMA2"))?);

    Ok(())
}
