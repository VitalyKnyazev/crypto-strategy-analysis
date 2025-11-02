mod dca;
pub use dca::Dca;

mod hodl;
pub use hodl::Hodl;

mod sma;
pub use sma::SmaPair;

mod sma2;
pub use sma2::Sma2Pair;

use crate::data::BinanceKline;
use yata::core::IndicatorResult;

pub trait BinanceIndicatorInstance {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult;
}
