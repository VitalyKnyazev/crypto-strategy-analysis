# Modernized, fixed and extended original project

First of all many thanks to original author Chiu Yat Tang `superoverflow` for such a nice demo project!

## Functional changes

- Added new strategies SMA (using yata) and SMA2 (custom implementation of SMA)
- Backtests were not actually running in parallel just with `async` methods and `futures::join`, now using `tokio::spawn` and `tokio::join` to have true concurrency
- Added `my_macros` crate to play with a procedural macros [log_duration] to estimate each backtest run time

## Code style changes

- Fixed NaiveDate initialization as its API changed since 2021
- Not using `unwrap`, instead added `anyhow` crate and returning `Result` with error propagation where possible `?`
- Not using `clone` (except `Arc::clone`)
- Not storing Kline iterator in trader struct (I guess originally it was done for academic purposes), passing next Kline into `next_trade_session` instead
- Using newer approach to modules instead of multiple mod.rs
- Update all crates to latest available version
- Tuned release build settings for performance
- Added `rustfmt.toml` to tune fmt formatter
- Fixed all lint errors including ones in *.md files
- Passing `&str` instead of `String` where appropriate
- Passing Kline slice instead of iterator where appropriate
- Using `if/let` instead of `match` where possible
- Using defaut [Default] trait implementation for DCA and HODL
- Fixed all clippy warnings
