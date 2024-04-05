use std::collections::HashMap;

use futures_util::{SinkExt, StreamExt};
use libs::market_data_module::{binance_data::ws_data, general_data};
use libs::mongo_engine::MongoEngine;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

pub struct WsDataEngine {
    url: String,
    symbols: Vec<String>,
    mongo_engine: MongoEngine,
    kline_records: HashMap<String, Vec<general_data::Kline>>,
}

impl Default for WsDataEngine {
    fn default() -> Self {
        Self {
            url: "wss://stream.binance.com".to_owned(),
            symbols: vec![],
            mongo_engine: MongoEngine::default(),
            kline_records: HashMap::new(),
        }
    }
}

impl WsDataEngine {
    pub async fn start(&mut self) {
        info!("WsDataEngine Start...");
        match tokio_tungstenite::connect_async(self.get_ws_url()).await {
            Ok((ws_stream, _)) => {
                let (mut write, mut read) = ws_stream.split();
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(msg) => match msg {
                            Message::Ping(ping) => match write.send(Message::Pong(ping)).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Error: {}", e);
                                }
                            },
                            Message::Text(data) => {
                                self.parse_ws_data(data).await;
                            }
                            _ => {}
                        },
                        Err(e) => {
                            error!("Error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        }
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

    fn get_ws_url(&self) -> String {
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

    async fn parse_ws_data(&mut self, data: String) {
        if let Ok(event) = serde_json::from_str::<ws_data::WsEvent>(&data) {
            let (stream, data) = (event.stream, event.data);
            let stream_parts: Vec<&str> = stream.split('@').collect();
            if stream_parts.len() != 2 {
                return;
            }
            let symbol = stream_parts[0];
            if stream == Self::get_kline_topic(symbol.to_string()) {
                if let Ok(ws_event_kline) = serde_json::from_value::<ws_data::WsEventKline>(data) {
                    let ws_kline = ws_event_kline.k;
                    if ws_kline.is_final() {
                        let kline = ws_kline.convert_to_standard_kline();
                        info!("receive {} kline: {:#?}", symbol, kline);
                        self.kline_records
                            .get_mut(symbol)
                            .unwrap()
                            .push(kline.clone());
                        if self.kline_records.get(symbol).unwrap().len() >= 1 {
                            match self.mongo_engine.fetch_latest_kline(symbol).await {
                                Ok(Some(latest_kline)) => {
                                    println!("mongo latest kline {:#?}", latest_kline);
                                    if self
                                        .kline_records
                                        .get(symbol)
                                        .unwrap()
                                        .get(0)
                                        .unwrap()
                                        .get_open_time()
                                        - latest_kline.get_open_time()
                                        == 5 * 60 * 1 * 1000
                                    {
                                        println!("insert klines: {:#?}", self.kline_records);
                                    }
                                }
                                Ok(None) => {
                                    println!("None");
                                }
                                Err(e) => {
                                    error!("Error: {}", e);
                                }
                            }
                        }
                    }
                }
            } else if stream == Self::get_depth_topic(symbol.to_string()) {
                if let Ok(ws_depth) = serde_json::from_value::<ws_data::WsDepth>(data) {
                    let depth = ws_depth.convert_to_standard_depth();
                    info!(
                        "{} receive depth mid price: {}",
                        symbol,
                        depth.get_mid_price()
                    );
                }
            }
        }
    }
}
