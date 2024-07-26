use crate::base_model::market_model::{depth_model::Depth, kline_model::Kline};


#[derive(Debug, Clone)]
pub enum MarketDataType {
    Kline(Kline),
    Depth(Depth),
}

#[derive(Debug, Clone)]
pub struct MarketData {
    symbol: String,
    data: MarketDataType,
}

impl MarketData {
    pub fn new(symbol: String, data: MarketDataType) -> Self {
        MarketData { symbol, data }
    }

    pub fn get_symbol(&self) -> &String {
        &self.symbol
    }

    pub fn get_kline(&self) -> Option<Kline> {
        match &self.data {
            MarketDataType::Kline(kline) => Some(*kline),
            _ => None,
        }
    }

    pub fn get_depth(&self) -> Option<Depth> {
        match &self.data {
            MarketDataType::Depth(depth) => Some(depth.clone()),
            _ => None,
        }
    }
}