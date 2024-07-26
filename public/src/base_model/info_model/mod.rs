use crate::base_enum::market_enums::MarketType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SymbolInfo {
    symbol: String,
    price_precision: i64,
    quantity_precision: i64,
    min_notional: f64,
    min_quantity: f64,
    max_quantity: f64,
}

impl SymbolInfo {
    pub fn new(
        symbol: String,
        price_precision: i64,
        quantity_precision: i64,
        min_notional: f64,
        min_quantity: f64,
        max_quantity: f64,
    ) -> Self {
        SymbolInfo {
            symbol,
            price_precision,
            quantity_precision,
            min_notional,
            min_quantity,
            max_quantity,
        }
    }

    pub fn get_symbol(&self) -> &String {
        &self.symbol
    }

    pub fn get_price_precision(&self) -> i64 {
        self.price_precision
    }

    pub fn get_quantity_precision(&self) -> i64 {
        self.quantity_precision
    }

    pub fn get_min_notional(&self) -> f64 {
        self.min_notional
    }

    pub fn get_min_quantity(&self) -> f64 {
        self.min_quantity
    }

    pub fn get_max_quantity(&self) -> f64 {
        self.max_quantity
    }
}



#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExchangeInfo {
    exchange: String,
    market_type: String,
    symbol_info: Vec<SymbolInfo>,
    rest_limit_rate: i64,
    server_time: i64,
}

impl ExchangeInfo {
    pub fn new(
        exchange: String,
        symbol_info: Vec<SymbolInfo>,
        market_type: MarketType,
        rest_limit_rate: i64,
        server_time: i64,
    ) -> Self {
        ExchangeInfo {
            exchange,
            symbol_info,
            market_type: market_type.get_market_type(),
            rest_limit_rate,
            server_time,
        }
    }

    pub fn get_exchange(&self) -> &String {
        &self.exchange
    }

    pub fn get_symbol_info(&self) -> &Vec<SymbolInfo> {
        &self.symbol_info
    }

    pub fn get_rest_limit_rate(&self) -> i64 {
        self.rest_limit_rate
    }

    pub fn get_server_time(&self) -> i64 {
        self.server_time
    }

    pub fn get_market_type(&self) -> &String {
        &self.market_type
    }

    pub fn get_symbol_info_map(&self, symbols: &Vec<String>) -> HashMap<String, SymbolInfo> {
        let mut res = HashMap::new();
        for symbol in symbols {
            for symbol_info in &self.symbol_info {
                if symbol_info.get_symbol().to_lowercase() == symbol.to_lowercase() {
                    res.insert(symbol.clone(), symbol_info.clone());
                }
            }
        }
        res
    }
}