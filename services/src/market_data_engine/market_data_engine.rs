use super::rest_data_engine::RestDataEngine;
use super::ws_data_engine::WsDataEngine;
use base_libs::market_data_module::general_data::MarketData;
use tokio::sync::broadcast::Sender;

pub struct MarketDataEngine {
    pub symbols: Vec<String>,
    ws_data_engine: WsDataEngine,
    rest_data_engine: RestDataEngine,
}

impl Default for MarketDataEngine {
    fn default() -> Self {
        Self {
            symbols: vec![],
            ws_data_engine: WsDataEngine::default(),
            rest_data_engine: RestDataEngine::default(),
        }
    }
}

impl MarketDataEngine {
    pub fn subscribe_symbols(&mut self, symbols: &Vec<String>) {
        self.symbols = symbols.to_vec();
        self.rest_data_engine.subscribe_symbols(symbols);
        self.ws_data_engine.subscribe_symbols(symbols);
    }

    pub async fn start(&mut self, tx: Sender<MarketData>) {
        self.rest_data_engine.update_exchange_info().await;
        self.rest_data_engine.start().await;
        // self.ws_data_engine.start().await;
        self.ws_data_engine.start_watch_send(tx).await;
    }
}
