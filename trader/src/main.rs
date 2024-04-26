use base_libs::tools::time_tools;
use base_libs::{base_strategy::base_strategy::StrategyContext, market_data_module::general_enum};
use services::market_data_engine::market_data_engine::MarketDataEngine;
use strategies::five_rsi_strategy;
use strategies::rma_strategy::RmaStrategy;
use tracing::info;
use tracing_subscriber;

use strategies::test_strategy::{self, TestStrategy};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // tracing_subscriber::fmt::init();
    // info!("start!");
    // let symbols = vec![
    //     "solusdt".to_string(),
    //     "avaxusdt".to_string(),
    //     "btcusdt".to_string(),
    //     "ethusdt".to_string(),
    //     "dogeusdt".to_string(),
    //     "bnbusdt".to_string(),
    //     "adausdt".to_string(),
    //     "tonusdt".to_string(),
    //     "1000shibusdt".to_string(),
    //     "dotusdt".to_string(),
    //     "linkusdt".to_string(),
    //     "trxusdt".to_string(),
    //     "maticusdt".to_string(),
    //     "nearusdt".to_string(),
    //     "uniusdt".to_string(),
    // ];

    // let mut m_engine = MarketDataEngine::default();
    // m_engine.subscribe_symbols(&symbols);
    // m_engine.start().await;
    let symbol = "btcusdt".to_string();
    let start_date_timestamp =
        time_tools::get_datetime_from_str("2024-01-01 00:00:00").timestamp_millis();

    // let test_strategy =
    //     TestStrategy::new("TestStrategy".to_string(), symbol.clone(), 33, 88, 8, 13);
    // let test_strategy =
    //     five_rsi_strategy::FiveRsiStrategy::new("five_rsi_strategy".to_string(), symbol.clone(), 3, 6);
    let test_strategy = RmaStrategy::new("RmaStrategy".to_string(), symbol.clone(), 16);
    let mut strategy = StrategyContext::new(
        vec![symbol.clone()],
        1000.0,
        start_date_timestamp,
        general_enum::Interval::Min15,
        20.0,
        true,
        Box::new(test_strategy),
    );
    strategy.start().await;
}
