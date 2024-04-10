use services::market_data_engine::market_data_engine::MarketDataEngine;
use tracing::info;
use tracing_subscriber;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("start!");
    let symbols = vec![
        "solusdt".to_string(),
        "avaxusdt".to_string(),
        // "btcusdt".to_string(),
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

    let mut m_engine = MarketDataEngine::default();
    m_engine.subscribe_symbols(&symbols);
    m_engine.start().await;
}
