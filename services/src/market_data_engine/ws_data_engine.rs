use std::collections::HashMap;

use public::tools::api_tools;
use public::base_model::api_model::MarketData;
use public::base_model::market_model::kline_model::Kline;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast::Sender;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct WsDataEngine {
    url: String,
    symbols: Vec<String>,
    // mongo_engine: MongoEngine,
    kline_records: HashMap<String, Vec<Kline>>,
}

impl Default for WsDataEngine {
    fn default() -> Self {
        Self {
            url: "wss://fstream.binance.com".to_owned(),
            symbols: vec![],
            // mongo_engine: MongoEngine::default(),
            kline_records: HashMap::new(),
        }
    }
}

impl WsDataEngine {
    pub async fn start_watch_send(&mut self, tx: Sender<MarketData>) {
        info!("WsDataEngine Start...");
        let ws_url = self.get_topic_url().clone();
        tokio::spawn(async move {
            match tokio_tungstenite::connect_async(ws_url).await {
                Ok((ws_stream, _)) => {
                    let (mut write, mut read) = ws_stream.split();
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(msg) => match msg {
                                Message::Ping(ping) => {
                                    match write.send(Message::Pong(ping)).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!("WsDataEngine Ping Error: {}", e);
                                        }
                                    }
                                }
                                Message::Text(data) => {
                                    if let Some(market_data) = api_tools::parse_market_data(&data)
                                    {
                                        match tx.send(market_data) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                error!("WsDataEngine Send Error: {}", e);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            },
                            Err(e) => {
                                error!("WsDataEngine ReadMsg Error: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("WsDataEngine Connect Error: {}", e);
                }
            }
        });
    }

    fn get_kline_topic(symbol: String) -> String {
        format!("{}@kline_5m", symbol)
    }

    fn get_depth_topic(symbol: String) -> String {
        format!("{}@depth5", symbol)
    }

    pub fn subscribe_symbols(&mut self, symbols: &Vec<String>) {
        self.symbols = symbols.to_vec();
        for s in self.symbols.iter() {
            self.kline_records.insert(s.clone(), vec![]);
        }
    }

    fn get_topic_url(&self) -> String {
        let topic = self
            .symbols
            .iter()
            .map(|x| {
                format!(
                    "{}/{}",
                    Self::get_kline_topic(x.clone()),
                    Self::get_depth_topic(x.clone())
                )
            })
            .collect::<Vec<String>>()
            .join("/");
        let full_topic = format!("{}/stream?streams={}", self.url, topic);
        full_topic
    }
}
