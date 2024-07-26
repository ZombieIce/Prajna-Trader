use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::general_enum;

pub struct CombineKline {
    klines: Vec<Kline>,
    interval: general_enum::Interval,
}

impl Default for CombineKline {
    fn default() -> Self {
        CombineKline {
            klines: vec![],
            interval: general_enum::Interval::Min5,
        }
    }
}

impl CombineKline {
    pub fn new(klines: Vec<Kline>, interval: general_enum::Interval) -> Self {
        CombineKline { klines, interval }
    }

    pub fn set_interval(&mut self, interval: general_enum::Interval) {
        self.interval = interval;
    }

    pub fn add(&mut self, kline: Kline) {
        self.klines.push(kline);
        if self.klines.len() > self.interval.get_divider() {
            self.klines.remove(0);
        }
    }

    pub fn get_kline(&mut self) -> Option<Kline> {
        if self.klines.len() == self.interval.get_divider() {
            let mut res = self.klines[0];
            for i in 1..self.klines.len() {
                res = res.combine(&self.klines[i]);
            }
            self.klines.clear();
            return Some(res);
        }
        None
    }
}

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

impl Default for Kline {
    fn default() -> Self {
        Kline {
            open_time: 0,
            close_time: 0,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
            number_of_trades: 0,
            active_buy_asset_volume: 0.0,
            active_buy_quote_volume: 0.0,
        }
    }
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

    pub fn get_open(&self) -> f64 {
        self.open
    }

    pub fn get_high(&self) -> f64 {
        self.high
    }

    pub fn get_low(&self) -> f64 {
        self.low
    }

    pub fn get_close(&self) -> f64 {
        self.close
    }

    pub fn get_volume(&self) -> f64 {
        self.volume
    }

    pub fn get_number_of_trades(&self) -> i64 {
        self.number_of_trades
    }

    pub fn get_active_buy_asset_volume(&self) -> f64 {
        self.active_buy_asset_volume
    }

    pub fn get_active_buy_quote_volume(&self) -> f64 {
        self.active_buy_quote_volume
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: general_enum::OrderSide,
    avg_price: f64,
    filled_qty: f64,
    order_type: general_enum::OrderType,
    cid: String,
    oid: String,
    timestamp: i64,
    status: general_enum::OrderStatus,
}

impl Default for Order {
    fn default() -> Self {
        Self {
            symbol: "".to_string(),
            price: 0.0,
            quantity: 0.0,
            side: general_enum::OrderSide::BUY,
            avg_price: 0.0,
            filled_qty: 0.0,
            order_type: general_enum::OrderType::Limit,
            cid: "".to_string(),
            oid: "".to_string(),
            status: general_enum::OrderStatus::New,
            timestamp: 0,
        }
    }
}

impl Order {
    pub fn new(
        symbol: &str,
        price: f64,
        quantity: f64,
        side: general_enum::OrderSide,
        order_type: general_enum::OrderType,
        avg_price: f64,
        filled_qty: f64,
        cid: &str,
        oid: &str,
        status: general_enum::OrderStatus,
        timestamp: i64,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            price,
            quantity,
            side,
            avg_price: avg_price,
            filled_qty: filled_qty,
            order_type,
            cid: cid.to_string(),
            oid: oid.to_string(),
            status,
            timestamp,
        }
    }

    pub fn get_symbol(&self) -> &str {
        &self.symbol
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }

    pub fn get_side(&self) -> &general_enum::OrderSide {
        &self.side
    }

    pub fn get_avg_price(&self) -> f64 {
        self.avg_price
    }

    pub fn get_filled_qty(&self) -> f64 {
        self.filled_qty
    }

    pub fn get_order_type(&self) -> &general_enum::OrderType {
        &self.order_type
    }

    pub fn get_cid(&self) -> &str {
        &self.cid
    }

    pub fn get_oid(&self) -> &str {
        &self.oid
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_status(&self) -> &general_enum::OrderStatus {
        &self.status
    }

    pub fn set_avg_price(&mut self, avg_price: f64) {
        self.avg_price = avg_price;
    }

    pub fn set_filled_qty(&mut self, filled_qty: f64) {
        self.filled_qty = filled_qty;
    }
    pub fn set_status(&mut self, status: general_enum::OrderStatus) {
        self.status = status;
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    symbol: String,
    price: f64,
    quantity: f64,
    break_even_price: f64,
    unrealized_pnl: f64,
    margin: f64,
    timestamp: i64,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            symbol: "".to_string(),
            price: 0.0,
            quantity: 0.0,
            break_even_price: 0.0,
            unrealized_pnl: 0.0,
            margin: 0.0,
            timestamp: 0,
        }
    }
}

impl Position {
    pub fn new(
        symbol: &str,
        price: f64,
        quantity: f64,
        break_even_price: f64,
        unrealized_pnl: f64,
        margin: f64,
        timestamp: i64,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            price,
            quantity,
            break_even_price,
            unrealized_pnl,
            margin,
            timestamp,
        }
    }

    pub fn get_symbol(&self) -> &str {
        &self.symbol
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_break_even_price(&self) -> f64 {
        self.break_even_price
    }

    pub fn get_unrealized_pnl(&self) -> f64 {
        self.unrealized_pnl
    }

    pub fn get_margin(&self) -> f64 {
        self.margin
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}
