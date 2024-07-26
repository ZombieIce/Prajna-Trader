use crate::market_data_module::{
    general_data,
    general_enum::{OrderSide, OrderStatus, OrderType},
};
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
    #[serde(rename = "a")]
    asks: Vec<Vec<String>>,
    #[serde(rename = "b")]
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

#[derive(Debug, Deserialize)]
pub struct WsOrder {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    quantity: String,
    #[serde(rename = "S")]
    side: String,
    #[serde(rename = "ap")]
    avg_price: String,
    #[serde(rename = "z")]
    filled_qty: String,
    #[serde(rename = "o")]
    order_type: String,
    #[serde(rename = "i")]
    oid: i64,
    #[serde(rename = "c")]
    cid: String,
    #[serde(rename = "T")]
    timestamp: i64,
    #[serde(rename = "X")]
    status: String,
}

impl WsOrder {
    pub fn convert_to_standard_order(&self) -> general_data::Order {
        general_data::Order::new(
            &self.symbol,
            self.price.parse().unwrap(),
            self.quantity.parse().unwrap(),
            OrderSide::parse_order_side(&self.side),
            OrderType::parse_order_type(&self.order_type),
            self.avg_price.parse().unwrap(),
            self.filled_qty.parse().unwrap(),
            &self.cid,
            &self.oid.to_string(),
            OrderStatus::parse_order_status(&self.status),
            self.timestamp,
        )
    }
}
