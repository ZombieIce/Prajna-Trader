use crate::market_data_module::general_data;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct WsKline {
    t: i64,
    #[serde(rename = "T")]
    et: i64,
    o: String,
    c: String,
    h: String,
    l: String,
    v: String,
    n: i64,
    x: bool,
    #[serde(rename = "V")]
    av: String,
    #[serde(rename = "Q")]
    q: String,
}

impl WsKline {
    pub fn convert_to_standard_kline(&self) -> general_data::Kline {
        general_data::Kline::new(
            self.t,
            self.et,
            self.o.parse().unwrap(),
            self.h.parse().unwrap(),
            self.l.parse().unwrap(),
            self.c.parse().unwrap(),
            self.v.parse().unwrap(),
            self.n,
            self.av.parse().unwrap(),
            self.q.parse().unwrap(),
        )
    }

    pub fn is_final(&self) -> bool {
        self.x
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct WsDepth {
    asks: Vec<Vec<String>>,
    bids: Vec<Vec<String>>,
}

impl WsDepth {
    pub fn convert_to_standard_depth(self) -> general_data::Depth {
        let asks = self
            .asks
            .iter()
            .map(|x| {
                general_data::PriceLevel::new(
                    x[0].parse::<f64>().unwrap(),
                    x[1].parse::<f64>().unwrap(),
                )
            })
            .collect();
        let bids = self
            .bids
            .iter()
            .map(|x| {
                general_data::PriceLevel::new(
                    x[0].parse::<f64>().unwrap(),
                    x[1].parse::<f64>().unwrap(),
                )
            })
            .collect();
        general_data::Depth::new(asks, bids)
    }
}

#[derive(Debug, Deserialize)]
pub struct WsEvent {
    pub stream: String,
    pub data: Value,
}
#[derive(Debug, serde::Deserialize)]
pub struct WsEventKline {
    pub k: WsKline,
}
