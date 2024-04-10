use super::market_data_module::general_data;
use super::market_data_module::general_enum;
use super::market_data_module::general_enum::Interval;
use super::mongo_engine::MongoEngine;
use chrono::prelude::*;
use polars::{df, frame::DataFrame};

fn resample_kline_data(
    klines: Vec<general_data::Kline>,
    interval: &general_enum::Interval,
) -> Vec<general_data::Kline> {
    let klines = cut_off_kline_data(klines, &interval);
    let res_kline = klines
        .chunks(interval.get_divider())
        .filter_map(|chunk| {
            if chunk.len() == interval.get_divider() {
                let cur_chunk = chunk.to_vec();
                let cur_r = cur_chunk
                    .iter()
                    .fold(cur_chunk[0], |acc, kline| acc.combine(kline));
                Some(cur_r)
            } else {
                None
            }
        })
        .collect();
    return res_kline;
}

fn cut_off_kline_data(
    klines: Vec<general_data::Kline>,
    interval: &general_enum::Interval,
) -> Vec<general_data::Kline> {
    if klines.len() == 0 {
        return vec![];
    } else {
        let mut res: Vec<general_data::Kline> = vec![];
        for i in 0..klines.len() {
            let cur_time = DateTime::from_timestamp_millis(klines[i].get_open_time()).unwrap();
            match interval {
                general_enum::Interval::Min5 => {
                    break;
                }
                general_enum::Interval::Min10 => {
                    if cur_time.minute() % 10 == 0 {
                        res = klines[i..].to_vec();
                        break;
                    }
                }
                general_enum::Interval::Min15 => {
                    if cur_time.minute() % 15 == 0 {
                        res = klines[i..].to_vec();
                        break;
                    }
                }
                general_enum::Interval::Min30 => {
                    if cur_time.minute() % 30 == 0 {
                        res = klines[i..].to_vec();
                        break;
                    }
                }
                general_enum::Interval::Hour1 => {
                    if cur_time.minute() == 0 {
                        res = klines[i..].to_vec();
                        break;
                    }
                }
                general_enum::Interval::Hour2 => {
                    if cur_time.minute() == 0 && cur_time.hour() % 2 == 0 {
                        res = klines[i..].to_vec();
                        break;
                    }
                }
                general_enum::Interval::Hour4 => {
                    if cur_time.minute() == 0 && cur_time.hour() % 4 == 0 {
                        res = klines[i..].to_vec();
                        break;
                    }
                }
                general_enum::Interval::Day => {
                    if cur_time.hour() == 0 && cur_time.minute() == 0 {
                        res = klines[i..].to_vec();
                        break;
                    }
                }
            }
        }
        return res;
    }
}

pub async fn fetch_klines(
    symbol: &str,
    start_date: i64,
    interval: &general_enum::Interval,
) -> Option<Vec<general_data::Kline>> {
    let mongo_engine = MongoEngine::default();
    match mongo_engine.fetch_klines(symbol, start_date).await {
        Ok(result) => {
            if let Some(klines) = result {
                Some(resample_kline_data(klines, &interval))
            } else {
                None
            }
        }
        Err(_) => {
            println!("error: fetch klines {symbol} failed");
            None
        }
    }
}

async fn get_kline_df(symbol: &str, start_data: DateTime<Utc>, interval: &Interval) -> DataFrame {
    let timestamp = start_data.timestamp_millis();

    let data = fetch_klines(symbol, timestamp, interval).await.unwrap();
    let dates: Vec<String> = data
        .iter()
        .map(|x| DateTime::<Utc>::from_timestamp_millis(x.get_open_time()).unwrap())
        .map(|x| x.to_string())
        .collect();
    let opens = data.iter().map(|x| x.get_open()).collect::<Vec<f64>>();
    let highs = data.iter().map(|x| x.get_high()).collect::<Vec<f64>>();
    let lows = data.iter().map(|x| x.get_low()).collect::<Vec<f64>>();
    let closes = data.iter().map(|x| x.get_close()).collect::<Vec<f64>>();
    let volumes = data.iter().map(|x| x.get_volume()).collect::<Vec<f64>>();
    let df: DataFrame = df!(
        "datetime" => dates,
        "open" => opens,
        "high" => highs,
        "low" => lows,
        "close" => closes,
        "volume" => volumes
    )
    .unwrap();
    df
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_klines() {
        let res = fetch_klines("btcusdt", 1712419200000, &general_enum::Interval::Hour4)
            .await
            .unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_polor_data_frame() {
        let start_data = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let df = get_kline_df("btcusdt", start_data, &Interval::Min30).await;
        println!("{:?}", df);
    }
}
