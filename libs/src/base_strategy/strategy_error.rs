// custom strategy error enum
use std::error::Error;
use std::fmt;
//

#[derive(Debug)]
pub enum StrategyError {
    InsufficientCashError(String),
}

impl fmt::Display for StrategyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StrategyError::InsufficientCashError(msg) => write!(f, "InsufficientCash: {}", msg),
        }
    }
}

impl Error for StrategyError {
    fn description(&self) -> &str {
        match self {
            StrategyError::InsufficientCashError(msg) => msg,
        }
    }
}
