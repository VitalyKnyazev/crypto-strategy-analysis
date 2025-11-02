use std::fs::File;
use std::io::prelude::Read;
use std::io::Cursor;
use std::iter::Iterator;

use yata::core::OHLCV;

use chrono::prelude::*;
use chrono::{Duration, NaiveDateTime, Utc};
use reqwest::{self};
use tempfile::tempfile;

use log::info;

use anyhow::{anyhow, Result};

fn is_current_month(year: i32, month: u32) -> bool {
    let now = Utc::now();
    let current_year = now.year();
    let current_month = now.month();
    year == current_year && month == current_month
}

fn binance_data_url(symbol: &str, interval: &str, year: i32, month: u32, day: u32) -> String {
    let folder = if is_current_month(year, month) { "daily" } else { "monthly" };
    let base_url = format!("https://data.binance.vision/data/spot/{folder}/klines");
    let file_name = match folder {
        "daily" => {
            format!("{symbol}-{interval}-{year}-{month:02}-{day:02}.zip")
        }
        "monthly" => format!("{symbol}-{interval}-{year}-{month:02}.zip"),
        _ => panic!("Not expected folder type"),
    };
    let url = format!("{base_url}/{symbol}/{interval}/{file_name}");
    url
}

async fn check_url_exists(url: &str) -> Result<bool> {
    let response = reqwest::get(url).await?;
    Ok(response.status().is_success())
}

async fn download_binance_data_to_file(url: &str, target: &mut File) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, target)?;
    Ok(())
}

fn read_zip_file(source: File) -> Result<String> {
    let mut archive = zip::ZipArchive::new(source)?;
    let mut data = archive.by_index(0)?;
    let mut buf = String::new();
    data.read_to_string(&mut buf)?;
    Ok(buf)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct BinanceKline {
    pub start_time: NaiveDateTime,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub end_time: NaiveDateTime,
}

impl OHLCV for BinanceKline {
    fn open(&self) -> f64 {
        self.open
    }
    fn close(&self) -> f64 {
        self.close
    }
    fn high(&self) -> f64 {
        self.high
    }
    fn low(&self) -> f64 {
        self.low
    }
    fn volume(&self) -> f64 {
        self.volume
    }
}

fn parse_binance_kline(data: &str) -> Result<Option<BinanceKline>> {
    if !data.contains(",") {
        return Ok(None);
    }
    let mut data = data.split(",");
    let start_time: i64 = data.next().ok_or(anyhow!("Missing start_time"))?.parse()?;
    let start_time = DateTime::from_timestamp(start_time / 1000, 0).ok_or(anyhow!("Invalid start_time timestamp"))?.naive_utc();
    let open: f64 = data.next().ok_or(anyhow!("Missing open"))?.parse()?;
    let close: f64 = data.next().ok_or(anyhow!("Missing close"))?.parse()?;
    let high: f64 = data.next().ok_or(anyhow!("Missing high"))?.parse()?;
    let low: f64 = data.next().ok_or(anyhow!("Missing low"))?.parse()?;
    let volume: f64 = data.next().ok_or(anyhow!("Missing volume"))?.parse()?;
    let end_time: i64 = data.next().ok_or(anyhow!("Missing end_time"))?.parse()?;
    let end_time = DateTime::from_timestamp(end_time / 1000, 0).ok_or(anyhow!("Invalid end_time timestamp"))?.naive_utc();

    let parsed = BinanceKline { start_time, open, close, high, low, volume, end_time };
    Ok(Some(parsed))
}

fn advance_date(current_date: NaiveDate) -> Result<NaiveDate> {
    let next_date = if !is_current_month(current_date.year(), current_date.month()) {
        if current_date.month() < 12 {
            NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1).ok_or(anyhow!("Invalid date"))?
        } else {
            NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1).ok_or(anyhow!("Invalid date"))?
        }
    } else {
        current_date + Duration::days(1)
    };
    Ok(next_date)
}

pub async fn get_kline_data(symbol: &str, interval: &str, from: NaiveDate, to: NaiveDate) -> Result<Vec<BinanceKline>> {
    let mut cur_date = from;
    let mut result: Vec<BinanceKline> = Vec::new();
    while cur_date < to {
        info!("fetching data for date: {cur_date}");

        let url = binance_data_url(symbol, interval, cur_date.year(), cur_date.month(), cur_date.day());
        let check = check_url_exists(&url).await?;
        if check {
            let mut temp_file = tempfile()?;
            download_binance_data_to_file(&url, &mut temp_file).await?;
            let content = read_zip_file(temp_file)?;
            for line in content.split("\n") {
                if let Some(data) = parse_binance_kline(line)? {
                    result.push(data)
                }
            }
        }
        cur_date = advance_date(cur_date)?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_timestamp(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Result<NaiveDateTime> {
        NaiveDate::from_ymd_opt(year, month, day).and_then(|d| d.and_hms_opt(hour, minute, second)).ok_or(anyhow!("cannot create timestamp"))
    }

    #[test]
    fn test_parse_binance_kline() -> Result<()> {
        let test_string = "1635739200000,4191.50000000,4320.00000000,4146.30000000,4302.93000000,88831.99690000,1635753599999,376834938.78850900,216236,45666.95420000,193846769.34658200,0";
        let result = parse_binance_kline(test_string)?;
        let expected = BinanceKline {
            start_time: create_timestamp(2021, 11, 01, 4, 0, 0)?,
            open: 4191.5,
            close: 4320.0,
            high: 4146.3,
            low: 4302.93,
            volume: 88831.9969,
            end_time: create_timestamp(2021, 11, 01, 7, 59, 59)?,
        };

        assert_eq!(result, Some(expected));

        Ok(())
    }
}
