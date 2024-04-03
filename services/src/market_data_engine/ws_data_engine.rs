use futures_util::{SinkExt, StreamExt};
use libs::market_data_module::binance_data::ws_data;
use serde::Deserialize;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Deserialize)]
struct WsEvent {
    stream: String,
    data: Value,
}
#[derive(Debug, serde::Deserialize)]
struct WsEventKline {
    k: ws_data::WsKline,
}

pub struct WsDataEngine {
    url: String,
    symbols: Vec<String>,
}

impl WsDataEngine {
    pub fn new(url: &str, symbols: Vec<String>) -> Self {
        Self {
            url: url.to_string(),
            symbols,
        }
    }

    pub async fn start(&self) {
        println!("Start ws data engine");
        match tokio_tungstenite::connect_async(self.get_ws_url()).await {
            Ok((ws_stream, _)) => {
                let (mut write, mut read) = ws_stream.split();
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(msg) => match msg {
                            Message::Ping(ping) => match write.send(Message::Pong(ping)).await {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("Error: {}", e);
                                }
                            },
                            Message::Text(data) => {
                                self.parse_ws_data(data);
                            }
                            _ => {}
                        },
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    fn get_kline_topic(symbol: String) -> String {
        format!("{}@kline_1m", symbol)
    }

    fn get_depth_topic(symbol: String) -> String {
        format!("{}@depth5", symbol)
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
        println!("{}", full_topic);
        full_topic
    }

    fn parse_ws_data(&self, data: String) {
        if let Ok(event) = serde_json::from_str::<WsEvent>(&data) {
            let (stream, data) = (event.stream, event.data);
            let stream_parts: Vec<&str> = stream.split('@').collect();
            if stream_parts.len() != 2 {
                return;
            }
            let symbol = stream_parts[0];
            if stream == Self::get_kline_topic(symbol.to_string()) {
                if let Ok(ws_event_kline) = serde_json::from_value::<WsEventKline>(data) {
                    let ws_kline = ws_event_kline.k;
                    if ws_kline.is_final() {
                        let kline = ws_kline.convert_to_standard_kline();
                        println!("receive {} kline: {:#?}", symbol, kline);
                    }
                }
            } else if stream == Self::get_depth_topic(symbol.to_string()) {
                if let Ok(ws_depth) = serde_json::from_value::<ws_data::WsDepth>(data) {
                    let depth = ws_depth.convert_to_standard_depth();
                    println!(
                        "{} receive depth mid price: {}",
                        symbol,
                        depth.get_mid_price()
                    );
                }
            }
        }
    }
}
