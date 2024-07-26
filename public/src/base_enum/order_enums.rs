use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    BUY,
    SELL,
}

impl OrderSide {
    pub fn string(&self) -> String {
        match self {
            OrderSide::BUY => "BUY".to_string(),
            OrderSide::SELL => "SELL".to_string(),
        }
    }

    pub fn parse_order_side(order_side: &str) -> OrderSide {
        match order_side {
            "BUY" => OrderSide::BUY,
            "SELL" => OrderSide::SELL,
            _ => panic!("Invalid order side"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
    TakeProfitMarket,
    StopMarket,
    Stop,
    TakeProfit,
    Liquidation,
    TrailingStopMarket,
}

impl OrderType {
    pub fn string(&self) -> String {
        match self {
            OrderType::Limit => "LIMIT".to_string(),
            OrderType::Market => "MARKET".to_string(),
            OrderType::TakeProfitMarket => "TAKE_PROFIT_MARKET".to_string(),
            OrderType::StopMarket => "STOP_MARKET".to_string(),
            OrderType::Stop => "STOP".to_string(),
            OrderType::TakeProfit => "TAKE_PROFIT".to_string(),
            OrderType::Liquidation => "LIQUIDATION".to_string(),
            OrderType::TrailingStopMarket => "TRAILING_STOP_MARKET".to_string(),
        }
    }

    pub fn parse_order_type(order_type: &str) -> OrderType {
        match order_type {
            "LIMIT" => OrderType::Limit,
            "MARKET" => OrderType::Market,
            "TAKE_PROFIT_MARKET" => OrderType::TakeProfitMarket,
            "STOP_MARKET" => OrderType::StopMarket,
            "STOP" => OrderType::Stop,
            "TAKE_PROFIT" => OrderType::TakeProfit,
            "LIQUIDATION" => OrderType::Liquidation,
            "TRAILING_STOP_MARKET" => OrderType::TrailingStopMarket,
            _ => panic!("Invalid order type"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
    ExpiredInMatch,
}

impl OrderStatus {
    pub fn parse_order_status(order_status: &str) -> OrderStatus {
        match order_status {
            "NEW" => OrderStatus::New,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            "EXPIRED_IN_MATCH" => OrderStatus::ExpiredInMatch,
            _ => panic!("Invalid order status"),
        }
    }

    pub fn string(&self) -> String {
        match self {
            OrderStatus::New => "NEW".to_string(),
            OrderStatus::PartiallyFilled => "PARTIALLY_FILLED".to_string(),
            OrderStatus::Filled => "FILLED".to_string(),
            OrderStatus::Canceled => "CANCELED".to_string(),
            OrderStatus::Rejected => "REJECTED".to_string(),
            OrderStatus::Expired => "EXPIRED".to_string(),
            OrderStatus::ExpiredInMatch => "EXPIRED_IN_MATCH".to_string(),
        }
    }
}