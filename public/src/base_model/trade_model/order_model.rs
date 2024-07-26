use crate::base_enum::order_enums::{OrderSide, OrderStatus, OrderType};
use crate::tools::math_tools;
use crate::tools::time_tools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    symbol: String,
    price: f64,
    qty: f64,
    side: OrderSide,
    avg_price: f64,
    filled_qty: f64,
    fee: f64,
    order_type: OrderType,
    cid: String,
    oid: String,
    timestamp: i64,
    status: OrderStatus,
}

impl Default for Order {
    fn default() -> Self {
        Self {
            symbol: "".to_string(),
            price: 0.0,
            qty: 0.0,
            side: OrderSide::BUY,
            avg_price: 0.0,
            filled_qty: 0.0,
            fee: 0.0,
            order_type: OrderType::Limit,
            cid: "".to_string(),
            oid: "".to_string(),
            timestamp: 0,
            status: OrderStatus::New,
        }
    }
}

impl Order {
    pub fn new(
        symbol: &str,
        price: f64,
        qty: f64,
        side: OrderSide,
        order_type: OrderType,
        avg_price: f64,
        filled_qty: f64,
        cid: &str,
        oid: &str,
        status: OrderStatus,
        timestamp: i64,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            price,
            qty,
            side,
            avg_price,
            filled_qty,
            fee: 0.0,
            order_type,
            cid: cid.to_string(),
            oid: oid.to_string(),
            timestamp,
            status,
        }
    }

    pub fn get_symbol(&self) -> &str {
        &self.symbol
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_qty(&self) -> f64 {
        self.qty
    }

    pub fn get_side(&self) -> OrderSide {
        self.side
    }

    pub fn get_avg_price(&self) -> f64 {
        self.avg_price
    }

    pub fn get_filled_qty(&self) -> f64 {
        self.filled_qty
    }

    pub fn get_order_type(&self) -> OrderType {
        self.order_type
    }

    pub fn get_cid(&self) -> &str {
        &self.cid
    }

    pub fn get_oid(&self) -> &str {
        &self.oid
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_status(&self) -> OrderStatus {
        self.status
    }

    pub fn set_symbol(&mut self, symbol: &str) {
        self.symbol = symbol.to_string();
    }

    pub fn set_price(&mut self, price: f64) {
        self.price = price;
    }

    pub fn set_qty(&mut self, qty: f64) {
        self.qty = qty;
    }

    pub fn set_side(&mut self, side: OrderSide) {
        self.side = side;
    }

    pub fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = timestamp;
    }

    pub fn set_avg_price(&mut self, avg_price: f64) {
        self.avg_price = avg_price;
    }

    pub fn set_filled_qty(&mut self, filled_qty: f64) {
        self.filled_qty = filled_qty;
    }

    pub fn set_fee(&mut self, fee: f64) {
        self.fee = fee;
    }

    pub fn get_fee(&self) -> f64 {
        self.fee
    }

    pub fn set_status(&mut self, status: OrderStatus) {
        self.status = status;
    }

    pub fn get_strategy_name(&self) -> String {
        let cid_parts: Vec<&str> = self.cid.split("_").collect();
        cid_parts[0].to_string()
    }

    pub fn string_order(&self) -> String {
        format!(
            "symbol: {}, price: {}, qty: {}, side: {}, cid: {}",
            self.symbol,
            self.price,
            self.qty,
            self.side.string(),
            self.cid,
        )
    }

    pub fn format_order(&mut self, px_precision: i64, qty_precision: i64) {
        self.set_price(math_tools::round_to_precision(
            self.get_price(),
            px_precision,
        ));
        self.set_qty(math_tools::round_to_precision(
            self.get_qty(),
            qty_precision,
        ));
    }
}

#[derive(Debug, Deserialize)]
pub struct OrderResponse {
    pub symbol: String,
    pub price: String,
    #[serde(rename = "origQty")]
    pub quantity: String,
    pub side: String,
    #[serde(rename = "avgPrice")]
    pub avg_price: String,
    #[serde(rename = "executedQty")]
    pub filled_qty: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "orderId")]
    pub order_id: i64,
    #[serde(rename = "clientOrderId")]
    pub cid: String,
    pub status: String,
    #[serde(rename = "updateTime")]
    pub timestamp: i64,
}

impl OrderResponse {
    pub fn order_response_into_order(&self) -> Order {
        Order::new(
            &self.symbol.to_lowercase(),
            self.price.parse().unwrap(),
            self.quantity.parse().unwrap(),
            OrderSide::parse_order_side(&self.side),
            OrderType::parse_order_type(&self.order_type),
            0.0,
            0.0,
            &self.cid,
            &self.order_id.to_string(),
            OrderStatus::parse_order_status(&self.status),
            self.timestamp,
        )
    }
}

pub trait OrderParser {
    fn parse_response_order(&self, data: String) -> Option<Order> {
        if let Ok(order_reponse) = serde_json::from_str::<OrderResponse>(&data) {
            Some(order_reponse.order_response_into_order())
        } else {
            None
        }
    }

    fn generate_cid(&self, symbol: &str, strategy: &str) -> String {
        let timestamp = time_tools::get_now_timestamp();
        format!("{}_{}_{}", strategy, symbol, timestamp)
    }

    fn parse_make_order(
        &self,
        symbol: String,
        px: f64,
        qty: f64,
        side: OrderSide,
        order_type: Option<OrderType>,
        strategy: Option<String>,
    ) -> Order {
        let mut cid = format!("test_{}_{}", symbol, time_tools::get_now_timestamp());
        if let Some(strategy) = strategy {
            cid = self.generate_cid(&symbol, &strategy);
        }
        let order = Order::new(
            &symbol,
            px,
            qty,
            side,
            order_type.unwrap_or(OrderType::Limit),
            0.0,
            0.0,
            &cid,
            "",
            OrderStatus::New,
            0,
        );
        order
    }
}
