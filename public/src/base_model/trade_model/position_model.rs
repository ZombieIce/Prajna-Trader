use serde::{Deserialize, Serialize};

use crate::base_enum::order_enums::OrderSide;
use crate::base_model::trade_model::order_model::Order;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    symbol: String,
    price: f64,
    quantity: f64,
    side: OrderSide,
    break_even_price: f64,
    leverage: f64,
    unrealized_pnl: f64,
    realized_pnl: f64,
    margin: f64,
    timestamp: i64,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            symbol: "".to_string(),
            price: 0.0,
            quantity: 0.0,
            side: OrderSide::BUY,
            break_even_price: 0.0,
            leverage: 100.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
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
        side: OrderSide,
        break_even_price: f64,
        leverage: f64,
        unrealized_pnl: f64,
        realized_pnl: f64,
        margin: f64,
        timestamp: i64,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            price,
            quantity,
            side,
            break_even_price,
            leverage,
            unrealized_pnl,
            realized_pnl,
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

    pub fn get_realized_pnl(&self) -> f64 {
        self.realized_pnl
    }

    pub fn get_margin(&self) -> f64 {
        self.margin
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_side(&self) -> OrderSide {
        self.side
    }

    pub fn update_order(&mut self, order: &Order) -> f64 {
        let tmp_margin = order.get_avg_price() * order.get_filled_qty() / self.leverage;
        self.timestamp = order.get_timestamp();
        if self.quantity == 0.0 {
            self.side = order.get_side();
            self.price = order.get_avg_price();
            self.quantity = order.get_filled_qty();
            self.margin = tmp_margin;
            return 0.0;
        } else {
            if self.side == order.get_side() {
                let new_quantity = self.quantity + order.get_filled_qty();
                self.price = (self.price * self.quantity
                    + order.get_avg_price() * order.get_filled_qty())
                    / new_quantity;
                self.quantity = new_quantity;
                self.margin += tmp_margin;
                return 0.0;
            } else {
                let remain_qty = order.get_filled_qty() - self.quantity;
                if remain_qty > 0.0 {
                    let delta_amt = (order.get_avg_price() - self.price) * self.quantity;
                    self.side = order.get_side();
                    self.price = order.get_avg_price();
                    self.quantity = remain_qty.abs();
                    self.margin = tmp_margin - self.margin;
                    if order.get_side() == OrderSide::BUY {
                        self.realized_pnl -= delta_amt;
                        return -delta_amt;
                    } else {
                        self.realized_pnl += delta_amt;
                        return delta_amt;
                    }
                } else if remain_qty < 0.0 {
                    let delta_amt = (order.get_avg_price() - self.price) * order.get_filled_qty();
                    self.quantity = remain_qty;
                    self.margin -= tmp_margin;
                    if order.get_side() == OrderSide::BUY {
                        self.realized_pnl -= delta_amt;
                        return -delta_amt;
                    } else {
                        self.realized_pnl += delta_amt;
                        return delta_amt;
                    }
                } else {
                    let delta_amt = (order.get_avg_price() - self.price) * self.quantity;
                    self.quantity = 0.0;
                    self.margin = 0.0;
                    if order.get_side() == OrderSide::BUY {
                        self.realized_pnl -= delta_amt;
                        return -delta_amt;
                    } else {
                        self.realized_pnl += delta_amt;
                        return delta_amt;
                    }
                }
            }
        }
    }

    pub fn update_market_price(&mut self, price: f64) {
        let tmp_unrealized_pnl = (price - self.price) * self.quantity;
        if self.side == OrderSide::BUY {
            self.unrealized_pnl = tmp_unrealized_pnl;
        } else {
            self.unrealized_pnl = -tmp_unrealized_pnl;
        }
    }

    pub fn sync_position(&mut self, position: &Position) {
        if self.quantity * position.quantity <= 0.0 {
            tracing::warn!("Position quantity is not matched");
            self.side = position.side;
        }
        self.price = position.price;
        self.quantity = position.quantity;
        self.break_even_price = position.break_even_price;
        self.unrealized_pnl = position.unrealized_pnl;
        self.margin = position.margin;
        self.timestamp = position.timestamp;
    }
}
