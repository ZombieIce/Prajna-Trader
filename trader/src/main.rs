use services::market_data_engine::ws_data_engine::WsDataEngine;

#[tokio::main]
async fn main() {
    let engine = WsDataEngine::new(
        "wss://stream.binance.com",
        vec!["btcusdt".to_string(), "ethusdt".to_string()],
    );
    engine.start().await;
}
