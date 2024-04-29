use super::{order::Order, strategy_error::StrategyError};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug)]
pub struct Portfolio {
    starting_cash: f64,
    available_cash: f64,
    freezed_cash: f64,
    total_value: f64,
    leverage_rate: f64,
    fee_rate: f64,
    fee: f64,
    unrealized_pnl: f64,
    realized_pnl: f64,
    positions: HashMap<String, Position>,
}

impl Portfolio {
    pub fn new(starting_cash: f64, leverage_rate: f64, symbols: Vec<String>) -> Self {
        Self {
            starting_cash,
            available_cash: starting_cash,
            freezed_cash: 0.0,
            leverage_rate: leverage_rate,
            total_value: starting_cash,
            fee_rate: 0.0005,
            fee: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            positions: symbols
                .into_iter()
                .map(|s| (s, Position::new()))
                .collect::<HashMap<String, Position>>(),
        }
    }

    fn update_cash_pnl(&mut self) {
        self.unrealized_pnl = 0.0;
        for (_, pos) in self.positions.iter() {
            self.unrealized_pnl += pos.unrealized_pnl;
        }
        self.total_value = self.available_cash + self.freezed_cash + self.unrealized_pnl;
        let epsilon = 1e-9;
        assert!(
            (self.total_value
                - (self.starting_cash + self.unrealized_pnl + self.realized_pnl - self.fee))
                .abs()
                < epsilon,
            "total_value is not equal to realized_pnl"
        );
        info!("total_value: {}", self.total_value);
    }

    pub fn update_market_price(&mut self, market_price: HashMap<String, f64>) {
        for (symbol, pos) in self.positions.iter_mut() {
            if let Some(price) = market_price.get(symbol) {
                pos.update_market_price(*price);
            }
        }
        self.update_cash_pnl();
    }

    pub fn make_orders(
        &mut self,
        orders: &mut HashMap<String, Order>,
    ) -> Result<(), StrategyError> {
        let mut realized_pnl_sum: f64 = 0.0;
        for (symbol, order) in orders.iter_mut() {
            if let Some(pos) = self.positions.get_mut(symbol) {
                let cur_fee = order.get_price() * order.get_qty().abs() * self.fee_rate;
                let cur_margin = order.get_price() * order.get_qty().abs() / self.leverage_rate;
                let prev_margin = pos.get_margin(self.leverage_rate);
                let new_margin: f64;
                if order.get_qty() * pos.get_qty() > 0.0 {
                    new_margin = prev_margin + cur_margin;
                } else {
                    if order.get_qty().abs() > pos.get_qty().abs() {
                        new_margin = cur_margin - prev_margin;
                    } else {
                        new_margin = prev_margin - cur_margin;
                    }
                }

                if cur_fee + new_margin - prev_margin > self.available_cash {
                    return Err(StrategyError::InsufficientCashError(
                        "Insufficient cash to make order".to_string(),
                    ));
                } else {
                    order.set_fee(cur_fee);
                    info!(
                        "{} MAKE ORDER at px: {}, qty: {}, fee: {}",
                        symbol,
                        order.get_price(),
                        order.get_qty(),
                        cur_fee
                    );

                    self.available_cash -= cur_fee + new_margin - prev_margin;
                    self.freezed_cash += new_margin - prev_margin;
                    order.set_fee(cur_fee);
                    realized_pnl_sum += pos.make_order(&order);
                    self.fee += cur_fee;
                }
            }
        }
        self.available_cash += realized_pnl_sum;
        self.realized_pnl += realized_pnl_sum;
        self.update_cash_pnl();

        Ok(())
    }

    pub fn get_pnl(&self) -> f64 {
        self.unrealized_pnl + self.realized_pnl - self.fee
    }

    pub fn get_unrealized_pnl(&self) -> f64 {
        self.unrealized_pnl
    }

    pub fn get_realized_pnl(&self) -> f64 {
        self.realized_pnl
    }

    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    pub fn get_total_value(&self) -> f64 {
        self.total_value
    }

    pub fn get_orders(&self, symbol: &str) -> Option<&Vec<Order>> {
        if let Some(pos) = self.positions.get(symbol) {
            Some(pos.get_orders())
        } else {
            None
        }
    }

    pub fn get_available_cash(&self) -> f64 {
        self.available_cash
    }

    pub fn get_starting_cash(&self) -> f64 {
        self.starting_cash
    }
}

#[derive(Debug)]
pub struct Position {
    avg_cost: f64,
    market_price: f64,
    quantity: f64,
    market_value: f64,
    unrealized_pnl: f64,
    realized_pnl: f64,
    orders: Vec<Order>,
}

impl Position {
    fn new() -> Self {
        Self {
            avg_cost: 0.0,
            market_price: 0.0,
            quantity: 0.0,
            market_value: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            orders: Vec::new(),
        }
    }
    fn get_margin(&self, leverage_rate: f64) -> f64 {
        self.avg_cost * self.quantity.abs() / leverage_rate
    }

    pub fn get_qty(&self) -> f64 {
        self.quantity
    }

    fn update_market_price(&mut self, price: f64) {
        self.market_price = price;
        self.market_value = self.quantity * self.market_price;
        self.unrealized_pnl = self.market_value - self.avg_cost * self.quantity;
    }

    fn make_order(&mut self, order: &Order) -> f64 {
        // offset
        self.orders.push(order.clone());
        if order.get_qty() * self.quantity < 0.0 {
            let epsilon = 1e-9;
            let offset = order.get_qty().abs() - self.quantity.abs();
            // close position
            if offset.abs() < epsilon {
                let cur_realized_pnl = (order.get_price() - self.avg_cost) * self.quantity;
                if order.get_qty() < 0.0 {
                    self.realized_pnl += cur_realized_pnl;
                } else {
                    self.realized_pnl -= cur_realized_pnl;
                }

                self.avg_cost = 0.0;
                self.market_price = order.get_price();
                self.quantity = 0.0;
                self.market_value = 0.0;
                self.unrealized_pnl = 0.0;
                return cur_realized_pnl;
            } else {
                let cur_realized_pnl: f64;
                if order.get_qty() < 0.0 {
                    if order.get_qty().abs() - self.quantity.abs() > epsilon {
                        cur_realized_pnl = (order.get_price() - self.avg_cost) * self.quantity;
                        self.avg_cost = order.get_price();
                    } else {
                        cur_realized_pnl =
                            (order.get_price() - self.avg_cost) * order.get_qty().abs();
                    }
                } else {
                    if order.get_qty().abs() - self.quantity.abs() > epsilon {
                        cur_realized_pnl = (order.get_price() - self.avg_cost) * self.quantity;
                        self.avg_cost = order.get_price();
                    } else {
                        cur_realized_pnl =
                            (order.get_price() - self.avg_cost) * order.get_qty().abs();
                    }
                }
                self.realized_pnl += cur_realized_pnl;
                self.market_price = order.get_price();
                self.quantity += order.get_qty();
                self.market_value = self.quantity * order.get_price();
                self.unrealized_pnl = self.market_value - self.avg_cost * self.quantity;
                return cur_realized_pnl;
            }
        } else {
            self.avg_cost = (self.avg_cost * self.quantity + order.get_price() * order.get_qty())
                / (self.quantity + order.get_qty());
            self.quantity += order.get_qty();
            self.market_value = self.quantity * order.get_price();
            self.unrealized_pnl = self.market_value - self.avg_cost * self.quantity;
            return 0.0;
        }
    }

    fn get_orders(&self) -> &Vec<Order> {
        &self.orders
    }
}
