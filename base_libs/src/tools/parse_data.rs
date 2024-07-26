use crate::base_model::order_model::Order;
use crate::market_data_module::binance_data::ws_data;
use crate::market_data_module::general_data::{MarketData, MarketDataType};

fn get_kline_topic(symbol: &str) -> String {
    format!("{}@kline_5m", symbol)
}

fn get_depth_topic(symbol: &str) -> String {
    format!("{}@depth5", symbol)
}

pub fn parse_market_data(data: &str) -> Option<MarketData> {
    if let Ok(event) = serde_json::from_str::<ws_data::WsEvent>(&data) {
        let (stream, data) = (event.stream, event.data);
        let stream_parts: Vec<&str> = stream.split('@').collect();
        if stream_parts.len() == 2 {
            let symbol = stream_parts[0];
            if stream == get_kline_topic(symbol) {
                if let Ok(ws_event_kline) = serde_json::from_value::<ws_data::WsEventKline>(data) {
                    let ws_kline = ws_event_kline.k;
                    if ws_kline.is_final() {
                        let kline = ws_kline.convert_to_standard_kline();
                        return Some(MarketData::new(
                            symbol.to_string(),
                            MarketDataType::Kline(kline),
                        ));
                    }
                }
            } else if stream == get_depth_topic(symbol) {
                if let Ok(ws_depth) = serde_json::from_value::<ws_data::WsDepth>(data) {
                    let depth = ws_depth.convert_to_standard_depth();
                    return Some(MarketData::new(
                        symbol.to_string(),
                        MarketDataType::Depth(depth),
                    ));
                }
            }
        }
    }
    None
}