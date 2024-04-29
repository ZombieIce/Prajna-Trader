use base_libs::market_data_module::general_data::MarketData;
use base_libs::tools::time_tools;
use base_libs::{base_strategy::base_strategy::StrategyContext, market_data_module::general_enum};
use plotters::data;
use services::market_data_engine::market_data_engine::MarketDataEngine;
use std::error;
use std::io;
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
    let file_appender = tracing_appender::rolling::daily("logs", "tracing.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let time_fmt =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]");
    let timer = OffsetTime::new(offset!(+8), time_fmt);
    tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_timer(timer)
        .init();
    // tracing_subscriber::fmt().with_timer(timer).init();

    info!("start!");
    let symbols = vec![
        "solusdt".to_string(),
        "avaxusdt".to_string(),
        "btcusdt".to_string(),
        "ethusdt".to_string(),
        "dogeusdt".to_string(),
        "bnbusdt".to_string(),
        "adausdt".to_string(),
        "tonusdt".to_string(),
        "1000shibusdt".to_string(),
        "dotusdt".to_string(),
        "linkusdt".to_string(),
        "trxusdt".to_string(),
        "maticusdt".to_string(),
        "nearusdt".to_string(),
        "uniusdt".to_string(),
    ];
    let symbol = "btcusdt".to_string();
    let start_date_timestamp =
        time_tools::get_datetime_from_str("2024-04-20 00:00:00").timestamp_millis();

    let mut m_engine = MarketDataEngine::default();
    m_engine.subscribe_symbols(&symbols);

    let (tx, _rx) = tokio::sync::broadcast::channel::<MarketData>(symbols.len());
    m_engine.start(tx.clone()).await;

    let mut rx = tx.subscribe();
    // let test_strategy = five_rsi_strategy::FiveRsiStrategy::new(
    //     "five_rsi_strategy".to_string(),
    //     symbol.clone(),
    //     3,
    //     6,
    // );
    let bl_strategy = bollinger_band_strategy::BollingerBandStrategy::new(
        "bollinger_band_strategy".to_string(),
        symbol.clone(),
        12,
        2.0,
    );
    let mut strategy = StrategyContext::new(
        vec![symbol.clone()],
        1000.0,
        start_date_timestamp,
        20.0,
        Box::new(bl_strategy),
    );
    strategy.init_trade().await;
    let handler = tokio::spawn(async move {
        while let Ok(data) = rx.recv().await {
            strategy.paper_trade(data).await;
        }
    });
    let _ = handler.await;
    // let test_strategy = SuperTrendStrategy::new("SuperTrend".to_string(), symbol.clone(), 16);
    // let mut strategy = StrategyContext::new(
    //     vec![symbol.clone()],
    //     1000.0,
    //     start_date_timestamp,
    //     20.0,
    //     Box::new(test_strategy),
    // );
    // strategy.init_trade().await;
    // let handler = tokio::spawn(async move {
    //     while let Ok(data) = rx.recv().await {
    //         strategy.real_trade(data).await;
    //     }
    // });
    // let _ = handler.await;

    // let test_strategy = RmaStrategy::new("RmaStrategy".to_string(), symbol.clone(), 16);
    // let mut strategy = StrategyContext::new(
    //     vec![symbol.clone()],
    //     1000.0,
    //     start_date_timestamp,
    //     general_enum::Interval::Min5,
    //     20.0,
    //     Box::new(test_strategy),
    // );
    // strategy.init_trade().await;

    // let symbol = "btcusdt".to_string();
    // let start_date_timestamp =
    //     time_tools::get_datetime_from_str("2024-04-1 00:00:00").timestamp_millis();

    // // let test_strategy =
    // //     TestStrategy::new("TestStrategy".to_string(), symbol.clone(), 33, 88, 8, 13);
    // let test_strategy = five_rsi_strategy::FiveRsiStrategy::new(
    //     "five_rsi_strategy".to_string(),
    //     symbol.clone(),
    //     3,
    //     6,
    // );
    // let mut strategy = StrategyContext::new(
    //     vec![symbol.clone()],
    //     1000.0,
    //     start_date_timestamp,
    //     20.0,
    //     Box::new(test_strategy),
    // );

    // strategy.start_backtest().await;
}
