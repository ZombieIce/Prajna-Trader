pub enum MarketType {
    SPOT,
    FUTURES,
}

impl MarketType {
    pub fn get_market_type(&self) -> String {
        match self {
            MarketType::SPOT => "spot".to_string(),
            MarketType::FUTURES => "futures".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub enum Interval {
    Min5,
    Min10,
    Min15,
    Min30,
    Hour1,
    Hour2,
    Hour4,
    Day,
}

impl Interval {
    pub fn get_divider(&self) -> usize {
        match self {
            Interval::Min5 => 1,
            Interval::Min10 => 2,
            Interval::Min15 => 3,
            Interval::Min30 => 6,
            Interval::Hour1 => 12,
            Interval::Hour2 => 24,
            Interval::Hour4 => 48,
            Interval::Day => 12 * 24,
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancled,
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
            "CANCELED" => OrderStatus::Cancled,
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
            OrderStatus::Cancled => "CANCELED".to_string(),
            OrderStatus::Rejected => "REJECTED".to_string(),
            OrderStatus::Expired => "EXPIRED".to_string(),
            OrderStatus::ExpiredInMatch => "EXPIRED_IN_MATCH".to_string(),
        }
    }
}
