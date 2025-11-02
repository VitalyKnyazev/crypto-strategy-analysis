# Making a small project is the best way to learn

- I am trying to download data from binance and work out how to manipulate the data/analyse the data using rust
- We try to find out some simple trading strategy on some big names in crypto

## Goal

- Compare HODL, DCA, SMA, MACD strategies for different coins
- Output Return, Max drawdown, drawdown recovery time

## Assumption

- holding for 1 year
- transaction fee 1.5%

## Learning Notes

- without `use std::io::prelude::Read;`, `file.read_to_string(&mut contents)` wont work. It need `Read` the read trait
- lots of `unwrap()`, need to think about how to get rid of them
- still not entirely sure why I can pass the String within a function outside the function, I thought it is out of scope
- want to find a more functional way to do stuff
- tokio/async good for fetch files -> thread is too expensive for such simple work
- tokio may have changed the mut ownership so i can pass mut files around from functions to functions?
- reqwest -> status code  -> is_success
- var match -> you may need struct { .. }
- mod is quite confusing at first <https://www.sheshbabu.com/posts/rust-module-system/>
- use `#[cfg(test)]` to do test driven development
- use `env_logger` for logging
- compare `unwrap` vs `expect` vs `?`
- I am trying to make `trader` being able to process any `IndicatorInstance`, i find it hard to write the type due to the `Size` trait constraint.
  I find you can actually use (Dynamically Size Type(DST))[https://docs.rs/yata/0.4.7/yata/prelude/dd/trait.IndicatorInstanceDyn.html]
- need life time 'a to determine borrow reference for functions
- Fixing dyn type in a trait was causing a bit of headache to me, at the end i found `self.indicator.as_mut()` and `Box` solved the issue of dynamic dispatched
- Notice why `&mut self.indicator` doesnt work but `self.indicator.as_mut()` works
- SuperTrait: something similar to inherit multiple interface in java
- Orphan rule: you need either the trait or the struct within your crate to impl it locally. You can
- While i am trying to make DCA Indicator, I have `impl has stricter requirements than trait` error. I was trying to feed a `BinanceKline:OHLCV` to `init<T: OHLVC>`. We are not allow to do this because `BinanceKline` is a subtype of `OHLCV`. When implementating the trait, we need to provide implementation of all possiblility but not possibility to a subset of types
- Solving the OHLVC problem seems to be tricky, because specialiation feature is not enabled in stable build. I can create a super trait to hold a custom indicator which consume ohlcv and time and the yata indicator, but it is a bit tricky on the return type and the method to be called.
- ta-rs is another library for TA and it offers flexibility for input type. if I cant make a good solution with yata, i may swap the library to ta-rs
- Rust vs cooking: we can think of stack/heap/generic/traits all these concept in terms of cooking.
  - think of a generic functions like making cocktail. we can have a generic trait for bartender that can prepare cocktail

   ```rust
   trait<S,T,C> Bartener where C: Cocktails{
     fn prepare_cocktail(spirit: S, T: water) -> C {
       ...
     }
   }

   ```

  - think of why do we need size? why do we need dyn/box

   ```rust
   trait Cashier {
     fn get_bill(currency)
   }
   ```

  - think of if one day we need to extend this library so that get_bill can handle crypto for example
  - how should we deal with it? newtype? impl new traits?
  - Tried to simplify the code by making a generic implementation but it wouldnt work because we dont have specialisation now

```rust
// Below will cause error due to implementation conflict. Unfortunately specialiation is not allow for stable release
impl<T:IndicatorInstance> BinanceIndicatorInstance for T {
     fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult {
         self.next(candle)
     }
}
```

- macro may help, need to look at it next

## Ideas

- Ideally, we should async pull the zip file, prepare the Kline data asychronously
- We can feed in the Kline to a analytics engine to calculate moving average
- We can use async stream and set up a channel for sinking the kline data, but for simplicity we choose to do async
- We would like to spawn thread to compare different strategies, while using async to pull data

## Integration Testing

- integration test - use `python -m http.server` on data folder

## Async

- `block_on` vs `await`
- `futures::join!`
