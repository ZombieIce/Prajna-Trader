use std::collections::HashMap;

use base_libs::market_data_module::general_data;
use base_libs::tools::parse_data;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast::Sender;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

pub struct WsDataEngine {
    url: String,
    symbols: Vec<String>,
    // mongo_engine: MongoEngine,
    kline_records: HashMap<String, Vec<general_data::Kline>>,
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
    // pub async fn start(&mut self) {
    //     info!("WsDataEngine Start...");
    //     match tokio_tungstenite::connect_async(self.get_ws_url()).await {
    //         Ok((ws_stream, _)) => {
    //             let (mut write, mut read) = ws_stream.split();
    //             while let Some(msg) = read.next().await {
    //                 match msg {
    //                     Ok(msg) => match msg {
    //                         Message::Ping(ping) => match write.send(Message::Pong(ping)).await {
    //                             Ok(_) => {}
    //                             Err(e) => {
    //                                 error!("Error: {}", e);
    //                             }
    //                         },
    //                         Message::Text(data) => {
    //                             self.parse_ws_data(data).await;
    //                         }
    //                         _ => {}
    //                     },
    //                     Err(e) => {
    //                         error!("Error: {}", e);
    //                     }
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             error!("Error: {}", e);
    //         }
    //     }
    // }

    pub async fn start_watch_send(&mut self, tx: Sender<general_data::MarketData>) {
        info!("WsDataEngine Start...");
        let ws_url = self.get_ws_url().clone();
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
                                    if let Some(kline) = parse_data::parse_market_data(&data) {
                                        match tx.send(kline) {
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

    // async fn insert_klines(&mut self, symbol: &str) {
    //     let insert_count = 1;
    //     let insert_kline = self.kline_records.get(symbol).unwrap();
    //     if insert_kline.len() >= insert_count {
    //         match self.mongo_engine.fetch_latest_kline(symbol).await {
    //             Ok(Some(last_kline)) => {
    //                 println!(
    //                     "insert_kline.get_open_time: {}",
    //                     insert_kline.get(0).unwrap().get_open_time()
    //                 );
    //                 println!("last_kline.get_open_time: {}", last_kline.get_open_time());
    //                 if insert_kline.get(0).unwrap().get_open_time() - last_kline.get_open_time()
    //                     == 5 * 60 * 1000 * insert_count as i64
    //                 {
    //                     println!(
    //                         "insert_kline.get_open_time: {}",
    //                         insert_kline.get(0).unwrap().get_open_time()
    //                     );

    //                     match self.mongo_engine.insert_kline(symbol, insert_kline).await {
    //                         Ok(_) => {
    //                             info!(
    //                                 "Insert klines {} count {} success!",
    //                                 symbol,
    //                                 insert_kline.len()
    //                             );
    //                         }
    //                         Err(e) => {
    //                             error!("Error: {}", e);
    //                         }
    //                     }
    //                 }
    //             }
    //             Ok(None) => match self.mongo_engine.insert_kline(symbol, insert_kline).await {
    //                 Ok(_) => {
    //                     info!(
    //                         "Insert klines {} count {} success!",
    //                         symbol,
    //                         insert_kline.len()
    //                     );
    //                 }
    //                 Err(e) => {
    //                     error!("Error: {}", e);
    //                 }
    //             },
    //             Err(e) => {
    //                 error!("Insert klines {} Error: {}", symbol, e);
    //             }
    //         }
    //     }
    // }

    // async fn parse_ws_data(&mut self, data: String) -> Option<Kline> {
    //     if let Ok(event) = serde_json::from_str::<ws_data::WsEvent>(&data) {
    //         let (stream, data) = (event.stream, event.data);
    //         let stream_parts: Vec<&str> = stream.split('@').collect();
    //         if stream_parts.len() != 2 {
    //             return None;
    //         }
    //         let symbol = stream_parts[0];
    //         if stream == Self::get_kline_topic(symbol.to_string()) {
    //             if let Ok(ws_event_kline) = serde_json::from_value::<ws_data::WsEventKline>(data) {
    //                 let ws_kline = ws_event_kline.k;
    //                 if ws_kline.is_final() {
    //                     let kline = ws_kline.convert_to_standard_kline();
    //                     info!("receive {} kline: {:#?}", symbol, kline);
    //                     self.kline_records
    //                         .get_mut(symbol)
    //                         .unwrap()
    //                         .push(kline.clone());
    //                     self.insert_klines(symbol).await;
    //                     return Some(kline);
    //                 }
    //             }
    //         } else if stream == Self::get_depth_topic(symbol.to_string()) {
    //             if let Ok(ws_depth) = serde_json::from_value::<ws_data::WsDepth>(data) {
    //                 let depth = ws_depth.convert_to_standard_depth();
    //                 info!(
    //                     "{} receive depth mid price: {}",
    //                     symbol,
    //                     depth.get_mid_price()
    //                 );
    //             }
    //         }
    //     }
    //     None
    // }
}
