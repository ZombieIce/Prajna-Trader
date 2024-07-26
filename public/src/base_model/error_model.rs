// custom strategy error enum
use std::error::Error;
use std::fmt;
use serde::Deserialize;
//

#[derive(Debug)]
pub enum StrategyError {
    InsufficientCashError(String),
    OrderQuantityError(String),
    OrderNotionalError(String),
    PlaceOrderError(String),
    BacktestError(String),
}

impl fmt::Display for StrategyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StrategyError::InsufficientCashError(msg) => write!(f, "InsufficientCash: {}", msg),
            StrategyError::OrderQuantityError(msg) => write!(f, "OrderQuantityError: {}", msg),
            StrategyError::OrderNotionalError(msg) => write!(f, "OrderNotionalError: {}", msg),
            StrategyError::PlaceOrderError(msg) => write!(f, "PlaceOrderError: {}", msg),
            StrategyError::BacktestError(msg) => write!(f, "BacktestError: {}", msg),
        }
    }
}

impl Error for StrategyError {
    fn description(&self) -> &str {
        match self {
            StrategyError::InsufficientCashError(msg) => msg,
            StrategyError::OrderQuantityError(msg) => msg,
            StrategyError::OrderNotionalError(msg) => msg,
            StrategyError::PlaceOrderError(msg) => msg,
            StrategyError::BacktestError(msg) => msg,
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct RequestError {
    pub code: i32,
    #[serde(rename = "msg")]
    pub message: String,
}

impl RequestError {
    pub fn parse_request_error_into_strategy_error(
        &self,
    ) -> StrategyError {
        StrategyError::PlaceOrderError(self.message.clone())
    }
}


