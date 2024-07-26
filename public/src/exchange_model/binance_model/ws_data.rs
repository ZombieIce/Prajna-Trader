use crate::base_enum::order_enums::*;
use crate::base_model::market_model::depth_model::{Depth, PriceLevel};
use crate::base_model::market_model::kline_model::Kline;
use crate::base_model::trade_model::order_model::Order;
use crate::base_model::trade_model::position_model::Position;

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
    pub fn convert_to_standard_kline(&self) -> Kline {
        Kline::new(
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
    pub fn convert_to_standard_depth(self) -> Depth {
        let asks = self
            .asks
            .iter()
            .map(|x| PriceLevel::new(x[0].parse::<f64>().unwrap(), x[1].parse::<f64>().unwrap()))
            .collect();
        let bids = self
            .bids
            .iter()
            .map(|x| PriceLevel::new(x[0].parse::<f64>().unwrap(), x[1].parse::<f64>().unwrap()))
            .collect();
        Depth::new(asks, bids)
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

#[derive(Debug, Deserialize, Clone)]
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
    #[serde(rename = "n")]
    fee: String,
}

impl WsOrder {
    pub fn convert_to_standard_order(&self) -> Order {
        let mut ord = Order::new(
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
        );
        ord.set_fee(self.fee.parse().unwrap());
        ord
    }
}

#[derive(Debug, Deserialize)]
pub struct WsPrivateData {
    e: String,
}

impl WsPrivateData {
    pub fn get_event(&self) -> String {
        self.e.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WsBalance {
    #[serde(rename = "a")]
    base: String,
    #[serde(rename = "wb")]
    balance: String,
}

impl WsBalance {
    pub fn get_base(&self) -> String {
        self.base.clone()
    }

    pub fn get_balance(&self) -> f64 {
        self.balance.parse().unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct WsAccountEvent {
    #[serde(rename = "E")]
    update_time: i64,
    #[serde(rename = "a")]
    data: WsDetail,
}

impl WsAccountEvent {
    pub fn get_update_time(&self) -> i64 {
        self.update_time
    }

    pub fn get_data(&self) -> WsDetail {
        self.data.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WsDetail {
    #[serde(rename = "B")]
    balance: Vec<WsBalance>,
    #[serde(rename = "P")]
    position: Vec<WsPostiion>,
}

impl WsDetail {
    pub fn get_balance(&self) -> Vec<WsBalance> {
        self.balance.clone()
    }

    pub fn get_position(&self) -> Vec<WsPostiion> {
        self.position.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WsPostiion {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "pa")]
    quantity: String,
    #[serde(rename = "ep")]
    price: String,
    #[serde(rename = "iw")]
    margin: String,
    #[serde(rename = "bep")]
    break_even_price: String,
    #[serde(rename = "up")]
    unrealized_pnl: String,
}

impl WsPostiion {
    pub fn convert_to_standard_position(&self, timestamp: i64) -> Position {
        let mut quantity: f64 = self.quantity.parse().unwrap();
        let mut side = OrderSide::BUY;
        if quantity < 0.0 {
            side = OrderSide::SELL;
            quantity = -quantity;
        }

        Position::new(
            &self.symbol,
            self.price.parse().unwrap(),
            quantity,
            side,
            self.break_even_price.parse().unwrap(),
            100.0,
            self.unrealized_pnl.parse().unwrap(),
            0.0,
            self.margin.parse().unwrap(),
            timestamp,
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct WsOrderEvent {
    #[serde(rename = "E")]
    update_time: i64,
    #[serde(rename = "o")]
    order: WsOrder,
}

impl WsOrderEvent {
    pub fn get_update_time(&self) -> i64 {
        self.update_time
    }

    pub fn get_order(&self) -> WsOrder {
        self.order.clone()
    }
}
