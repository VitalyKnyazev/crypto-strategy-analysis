mod generic_trader;
pub use generic_trader::{GenericTrader, StakeSize, TradingFee};

mod macd_trader;
pub use macd_trader::MACDTrader;

mod hodl_trader;
pub use hodl_trader::HODLTrader;

mod dca_trader;
pub use dca_trader::DCATrader;

mod sma_trader;
pub use sma_trader::SMATrader;

mod sma2_trader;
pub use sma2_trader::SMA2Trader;
