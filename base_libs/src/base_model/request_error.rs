use serde::Deserialize;
use crate::base_strategy::strategy_error::StrategyError;

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
