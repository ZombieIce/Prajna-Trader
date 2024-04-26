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