use serde::{Deserialize, Serialize};

use super::general_enum;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Kline {
    open_time: i64,
    close_time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    number_of_trades: i64,
    active_buy_asset_volume: f64,
    active_buy_quote_volume: f64,
}

impl Kline {
    pub fn new(
        open_time: i64,
        close_time: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        number_of_trades: i64,
        active_buy_asset_volume: f64,
        active_buy_quote_volume: f64,
    ) -> Self {
        Kline {
            open_time,
            close_time,
            open,
            high,
            low,
            close,
            volume,
            number_of_trades,
            active_buy_asset_volume,
            active_buy_quote_volume,
        }
    }

    pub fn combine(&self, other: &Kline) -> Kline {
        Kline {
            open_time: self.open_time,
            close_time: other.close_time,
            open: self.open,
            high: self.high.max(other.high),
            low: self.low.min(other.low),
            close: other.close,
            volume: self.volume + other.volume,
            number_of_trades: self.number_of_trades + other.number_of_trades,
            active_buy_asset_volume: self.active_buy_asset_volume + other.active_buy_asset_volume,
            active_buy_quote_volume: self.active_buy_quote_volume + other.active_buy_quote_volume,
        }
    }

    pub fn get_open_time(&self) -> i64 {
        self.open_time
    }

    pub fn get_close_time(&self) -> i64 {
        self.close_time
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PriceLevel {
    price: f64,
    quantity: f64,
}

impl PriceLevel {
    pub fn new(price: f64, quantity: f64) -> Self {
        PriceLevel { price, quantity }
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }
}

#[derive(Debug)]
pub struct Depth {
    asks: Vec<PriceLevel>,
    bids: Vec<PriceLevel>,
}

impl Depth {
    pub fn new(asks: Vec<PriceLevel>, bids: Vec<PriceLevel>) -> Self {
        Depth { asks, bids }
    }

    pub fn get_asks(&self) -> &Vec<PriceLevel> {
        &self.asks
    }

    pub fn get_bids(&self) -> &Vec<PriceLevel> {
        &self.bids
    }

    pub fn get_best_bid(&self) -> PriceLevel {
        self.bids[0].clone()
    }

    pub fn get_best_ask(&self) -> PriceLevel {
        self.asks.last().unwrap().clone()
    }

    pub fn get_mid_price(&self) -> f64 {
        (self.get_best_bid().get_price() + self.get_best_ask().get_price()) / 2.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SymbolInfo {
    symbol: String,
    price_precision: i64,
    quantity_precision: i64,
}

impl SymbolInfo {
    pub fn new(symbol: String, price_precision: i64, quantity_precision: i64) -> Self {
        SymbolInfo {
            symbol,
            price_precision,
            quantity_precision,
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
        market_type: general_enum::MarketType,
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
}
