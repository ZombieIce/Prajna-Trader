use crate::base_enum::order_enums::{OrderSide, OrderStatus, OrderType};
use crate::base_model::trade_model::{order_model::Order, position_model::Position};
use serde::Deserialize;
use serde_json::Value;

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

#[derive(Debug, Deserialize)]
pub struct PositionResponse {
    pub symbol: String,
    #[serde(rename = "entryPrice")]
    pub price: String,
    #[serde(rename = "positionAmt")]
    pub quantity: String,
    #[serde(rename = "markPrice")]
    pub market_price: String,
    #[serde(rename = "leverage")]
    pub leverage: String,
    #[serde(rename = "positionSide")]
    pub side: String,
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
        let mut side = OrderSide::BUY;
        let market_price: f64 = self.market_price.parse().unwrap();
        let price: f64 = self.price.parse().unwrap();
        let unrealized_pnl: f64 = self.unrealized_pnl.parse().unwrap();
        if market_price < price && unrealized_pnl > 0.0 {
            side = OrderSide::SELL;
        } else if market_price > price && unrealized_pnl < 0.0 {
            side = OrderSide::BUY;
        }
        Position::new(
            &self.symbol.to_lowercase(),
            price,
            self.quantity.parse().unwrap(),
            side,
            self.break_even_price.parse().unwrap(),
            self.leverage.parse().unwrap(),
            unrealized_pnl,
            0.0,
            self.margin.parse().unwrap(),
            self.timestamp,
        )
    }
}
