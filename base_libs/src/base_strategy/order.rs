use crate::market_data_module::general_data::{Kline, SymbolInfo};
use std::collections::HashMap;

use crate::tools::common_tools;

use super::{common_module::TargetPosition, portfolio::Portfolio, strategy_error::StrategyError};

#[derive(Debug, Clone)]
pub struct Order {
    timestamp: i64,
    price: f64,
    qty: f64,
    fee: f64,
}

impl Order {
    pub fn new(timestamp: i64, price: f64, qty: f64) -> Self {
        Self {
            timestamp,
            price,
            qty,
            fee: 0.0,
        }
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_qty(&self) -> f64 {
        self.qty
    }

    pub fn set_price(&mut self, price: f64) {
        self.price = price;
    }

    pub fn set_qty(&mut self, qty: f64) {
        self.qty = qty;
    }

    pub fn set_fee(&mut self, fee: f64) {
        self.fee = fee;
    }

    pub fn get_fee(&self) -> f64 {
        self.fee
    }

    pub fn format_order(&mut self, px_precision: i64, qty_precision: i64) {
        self.set_price(common_tools::round_to_precision(
            self.get_price(),
            px_precision,
        ));
        self.set_qty(common_tools::round_to_precision(
            self.get_qty(),
            qty_precision,
        ));
    }
}

pub struct OrderParse {
    symbol_infos: HashMap<String, SymbolInfo>,
}

impl Default for OrderParse {
    fn default() -> Self {
        Self {
            symbol_infos: HashMap::new(),
        }
    }
}

impl OrderParse {
    pub fn new(symbol_infos: HashMap<String, SymbolInfo>) -> Self {
        Self { symbol_infos }
    }

    pub fn set_symbol_infos(&mut self, symbol_infos: HashMap<String, SymbolInfo>) {
        self.symbol_infos = symbol_infos;
    }

    fn check_order(&self, symbol: &str, order: &Order) -> Result<(), StrategyError> {
        let symbol_info = self.symbol_infos.get(symbol).unwrap();
        if order.get_qty().abs() < symbol_info.get_min_quantity() {
            return Err(StrategyError::OrderQuantityError(format!(
                "order quantity {} is too small with min_qty: {}",
                order.get_qty(),
                symbol_info.get_min_quantity()
            )));
        }
        if order.get_qty().abs() * order.get_price() < symbol_info.get_min_notional() {
            return Err(StrategyError::OrderNotionalError(format!(
                "order notional {} is too small with min_notional: {}",
                order.get_qty() * order.get_price(),
                symbol_info.get_min_notional()
            )));
        }
        Ok(())
    }

    fn common_convert_order(
        &self,
        orders: &HashMap<String, TargetPosition>,
        klines: &HashMap<String, Kline>,
        portfolio: &Portfolio,
        is_back_test: bool,
    ) -> Result<HashMap<String, Order>, StrategyError> {
        let mut res: HashMap<String, Order> = HashMap::new();
        for (symbol, target_position) in orders {
            let target_position = target_position.get_position();
            let price = if is_back_test {
                klines.get(symbol).unwrap().get_open()
            } else {
                klines.get(symbol).unwrap().get_close()
            };
            let timestamp = if is_back_test {
                klines.get(symbol).unwrap().get_open_time()
            } else {
                klines.get(symbol).unwrap().get_close_time()
            };

            let symbol_info = self.symbol_infos.get(symbol).unwrap();
            if let Some(curretn_position) = portfolio.get_position(symbol) {
                let mut order = Order::new(
                    timestamp,
                    price,
                    target_position - curretn_position.get_qty(),
                );
                order.format_order(
                    symbol_info.get_price_precision(),
                    symbol_info.get_quantity_precision(),
                );
                match self.check_order(symbol, &order) {
                    Ok(_) => res.insert(symbol.clone(), order),
                    Err(e) => return Err(e),
                };
            } else {
                let mut order = Order::new(timestamp, price, target_position);
                order.format_order(
                    symbol_info.get_price_precision(),
                    symbol_info.get_quantity_precision(),
                );
                match self.check_order(symbol, &order) {
                    Ok(_) => res.insert(symbol.clone(), order),
                    Err(e) => return Err(e),
                };
            }
        }
        Ok(res)
    }

    pub fn convert_backetst_order(
        &self,
        orders: &HashMap<String, TargetPosition>,
        klines: &HashMap<String, Kline>,
        portfolio: &Portfolio,
    ) -> Result<HashMap<String, Order>, StrategyError> {
        self.common_convert_order(orders, klines, portfolio, true)
    }

    pub fn convert_paper_order(
        &self,
        orders: &HashMap<String, TargetPosition>,
        klines: &HashMap<String, Kline>,
        portfolio: &Portfolio,
    ) -> Result<HashMap<String, Order>, StrategyError> {
        self.common_convert_order(orders, klines, portfolio, false)
    }
}
