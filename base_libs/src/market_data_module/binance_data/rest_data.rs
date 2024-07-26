use serde::Deserialize;
use serde_json::Value;

use crate::market_data_module::{
    general_data::{Order, Position},
    general_enum,
};

#[derive(Debug, Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Value,
    #[serde(rename = "rateLimits")]
    pub rate_limits: Value,
    #[serde(rename = "serverTime")]
    pub server_time: i64,
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
    pub fn convert_into_order(&self) -> Order {
        Order::new(
            &self.symbol.to_lowercase(),
            self.price.parse().unwrap(),
            self.quantity.parse().unwrap(),
            general_enum::OrderSide::parse_order_side(&self.side),
            general_enum::OrderType::parse_order_type(&self.order_type),
            0.0,
            0.0,
            &self.cid,
            &self.order_id.to_string(),
            general_enum::OrderStatus::parse_order_status(&self.status),
            self.timestamp,
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct PositionResponse {
    pub symbol: String,
    #[serde(rename = "entryPrice")]
    pub price: String,
    #[serde(rename = "positionAmt")]
    pub quantity: String,
    #[serde(rename = "breakEvenPrice")]
    pub break_even_price: String,
    #[serde(rename = "unRealizedProfit")]
    pub unrealized_pnl: String,
    #[serde(rename = "notional")]
    pub margin: String,
    #[serde(rename = "updateTime")]
    pub timestamp: i64,
}

impl PositionResponse {
    pub fn convert_into_position(&self) -> Position {
        Position::new(
            &self.symbol.to_lowercase(),
            self.price.parse().unwrap(),
            self.quantity.parse().unwrap(),
            self.break_even_price.parse().unwrap(),
            self.unrealized_pnl.parse().unwrap(),
            self.margin.parse().unwrap(),
            self.timestamp,
        )
    }
}
