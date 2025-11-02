use anyhow::Ok;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;

pub struct Account {
    pub available_fund: f64,
    pub position: Position,
    pub profit_and_loss_history: Vec<TimeValue>,
    pub trade_history: Vec<Trade>,
}

#[derive(Debug, PartialEq)]
pub struct TimeValue {
    timestamp: NaiveDateTime,
    realised_pnl: f64,
    unrealised_pnl: f64,
}

#[derive(Debug, PartialEq)]
pub struct Position {
    pub quantity: f64,
    pub cost: f64,
}

#[derive(Debug, PartialEq)]
pub struct Trade {
    timestamp: NaiveDateTime,
    buy_sell_indicator: BuySellIndicator,
    quantity: f64,
    price: f64,
    fee: f64,
}

#[derive(Debug, PartialEq)]
enum BuySellIndicator {
    Buy,
    Sell,
}

impl Account {
    pub fn new(fund: f64, initial_position: Position, start_timestamp: NaiveDateTime) -> Self {
        let initial_pnl = TimeValue { timestamp: start_timestamp, realised_pnl: 0., unrealised_pnl: 0. };
        Self { available_fund: fund, position: initial_position, profit_and_loss_history: vec![initial_pnl], trade_history: Vec::new() }
    }

    fn average_cost(&self, quantity: f64, price: f64) -> f64 {
        (self.position.quantity * self.position.cost + quantity * price) / (self.position.quantity + quantity)
    }

    pub fn open(&mut self, timestamp: NaiveDateTime, quantity: f64, price: f64, fee: f64) {
        self.position.cost = self.average_cost(quantity, price);
        self.position.quantity += quantity;
        self.available_fund -= price * quantity + fee;

        self.trade_history.push(Trade { timestamp, buy_sell_indicator: BuySellIndicator::Buy, quantity, price, fee });
    }

    pub fn close(&mut self, timestamp: NaiveDateTime, quantity: f64, price: f64, fee: f64) -> Result<()> {
        let last_pnl = self.profit_and_loss_history.last().ok_or(anyhow!("No PnL history"))?;
        let current_pnl = quantity * (price - self.position.cost);
        let realised_pnl = last_pnl.realised_pnl + current_pnl;
        let unrealised_pnl = last_pnl.unrealised_pnl - current_pnl;
        let new_pnl = TimeValue { timestamp, realised_pnl, unrealised_pnl };
        self.profit_and_loss_history.push(new_pnl);

        self.position.quantity -= quantity;
        self.available_fund += price * quantity - fee;

        self.trade_history.push(Trade { timestamp, buy_sell_indicator: BuySellIndicator::Sell, quantity, price, fee });

        Ok(())
    }

    pub fn mark_to_market(&mut self, timestamp: NaiveDateTime, closing_price: f64) -> Result<()> {
        let last_pnl = self.profit_and_loss_history.last().ok_or(anyhow!("No PnL history"))?;
        let unrealised_pnl = self.position.quantity * (closing_price - self.position.cost);
        let new_pnl = TimeValue { timestamp, unrealised_pnl, realised_pnl: last_pnl.realised_pnl };
        self.profit_and_loss_history.push(new_pnl);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;
    use chrono::NaiveDate;

    fn create_timestamp(year: i32, month: u32, day: u32) -> Result<NaiveDateTime> {
        NaiveDate::from_ymd_opt(year, month, day).and_then(|d| d.and_hms_opt(0, 0, 0)).ok_or(anyhow!("Cannot create timestamp"))
    }

    #[test]
    fn test_has_position() -> Result<()> {
        let initial_position = Position { quantity: 123.1, cost: 10.0 };
        let start_timestamp = create_timestamp(2021, 9, 1)?;
        let account = Account::new(1000.0, initial_position, start_timestamp);
        assert_eq!(account.position.quantity, 123.1);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<()> {
        let initial_position = Position { quantity: 100.0, cost: 10.0 };
        let start_timestamp = create_timestamp(2021, 9, 1)?;
        let mut account = Account::new(7000.0, initial_position, start_timestamp);
        let timestamp = create_timestamp(2021, 10, 31)?;
        account.open(timestamp, 100.0, 20.0, 0.02);
        assert_eq!(account.position, Position { cost: 15.0, quantity: 200.0 });
        assert_eq!(4999.98, account.available_fund);
        assert_eq!(vec![Trade { timestamp: create_timestamp(2021, 10, 31)?, buy_sell_indicator: BuySellIndicator::Buy, quantity: 100.0, price: 20.0, fee: 0.02 }], account.trade_history);

        Ok(())
    }

    #[test]
    fn test_close() -> Result<()> {
        let initial_position = Position { quantity: 100.0, cost: 10.0 };
        let start_timestamp = create_timestamp(2021, 9, 1)?;
        let mut account = Account::new(1000.0, initial_position, start_timestamp);
        let timestamp = create_timestamp(2021, 10, 31)?;
        account.close(timestamp, 50.0, 20.0, 0.02)?;
        assert_eq!(account.position, Position { cost: 10.0, quantity: 50.0 });
        assert_eq!(account.available_fund, 1999.98);
        assert_eq!(vec![Trade { timestamp: create_timestamp(2021, 10, 31)?, buy_sell_indicator: BuySellIndicator::Sell, quantity: 50.0, price: 20.0, fee: 0.02 }], account.trade_history);

        Ok(())
    }

    #[test]
    fn test_mark_to_market() -> Result<()> {
        let initial_position = Position { quantity: 100.0, cost: 10.0 };
        let start_timestamp = create_timestamp(2021, 9, 1)?;
        let mut account = Account::new(5000.0, initial_position, start_timestamp);
        let timestamp = create_timestamp(2021, 10, 31)?;
        account.mark_to_market(timestamp.clone(), 20.0)?;

        let latest_pnl = account.profit_and_loss_history.last().ok_or(anyhow!("No PnL history"))?;
        assert_eq!(*latest_pnl, TimeValue { timestamp, realised_pnl: 0., unrealised_pnl: 1000. });

        Ok(())
    }
}
