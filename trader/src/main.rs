use plotters::data;
use public::base_model::api_model::MarketData;
use public::tools::time_tools;
use services::market_data_engine::market_data_engine::MarketDataEngine;
use strategies::rma_strategy::RmaStrategy;
use strategies::super_trend_strategy::SuperTrendStrategy;
use strategies::{bollinger_band_strategy, five_rsi_strategy};
use time::macros::{format_description, offset};
use tracing::info;
use tracing_subscriber;
use tracing_subscriber::fmt::time::OffsetTime;

use strategies::test_strategy::{self, TestStrategy};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // let file_appender = tracing_appender::rolling::daily("logs", "tracing.log");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // let time_fmt =
    //     format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]");
    // let timer = OffsetTime::new(offset!(+8), time_fmt);
    // tracing_subscriber::fmt()
    //     .with_writer(io::stdout)
    //     .with_writer(non_blocking)
    //     .with_ansi(false)
    //     .with_timer(timer)
    //     .init();
    tracing_subscriber::fmt().init();

    info!("start!");
    let symbols = vec![
        "dydxusdt".to_string(),
        // "solusdt".to_string(),
        // "avaxusdt".to_string(),
        // "ethusdt".to_string(),
        // "dogeusdt".to_string(),
        // "bnbusdt".to_string(),
        // "adausdt".to_string(),
        // "tonusdt".to_string(),
        // "1000shibusdt".to_string(),
        // "dotusdt".to_string(),
        // "linkusdt".to_string(),
        // "trxusdt".to_string(),
        // "maticusdt".to_string(),
        // "nearusdt".to_string(),
        // "uniusdt".to_string(),
    ];
    // let symbol = "btcusdt".to_string();
    // let start_date_timestamp =
    //     time_tools::get_datetime_from_str("2024-05-5 00:00:00").timestamp_millis();

    let mut m_engine = MarketDataEngine::default();
    m_engine.subscribe_symbols(&symbols);

    let (tx, _rx) = tokio::sync::broadcast::channel::<MarketData>(symbols.len());
    m_engine.start(tx.clone()).await;
}
