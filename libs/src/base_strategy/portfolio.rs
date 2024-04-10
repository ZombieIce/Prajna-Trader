use super::strategy_error::StrategyError;
use std::{collections::HashMap, error::Error};

pub struct Order {
    price: f64,
    qty: f64,
}

impl Order {
    pub fn new(price: f64, qty: f64) -> Self {
        Self { price, qty }
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_qty(&self) -> f64 {
        self.qty
    }
}

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
        )
    }

    pub fn update_market_price(&mut self, market_price: HashMap<String, f64>) {
        for (symbol, pos) in self.positions.iter_mut() {
            if let Some(price) = market_price.get(symbol) {
                pos.update_market_price(*price);
            }
        }
        self.update_cash_pnl();
    }

    pub fn make_orders(&mut self, orders: &HashMap<String, Order>) -> Result<(), StrategyError> {
        let mut realized_pnl_sum: f64 = 0.0;
        for (symbol, order) in orders.iter() {
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
                    self.available_cash -= cur_fee + new_margin - prev_margin;
                    self.freezed_cash += new_margin - prev_margin;
                    realized_pnl_sum += pos.make_order(order);
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
        self.unrealized_pnl + self.realized_pnl
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
}

#[derive(Debug)]
pub struct Position {
    avg_cost: f64,
    market_price: f64,
    quantity: f64,
    market_value: f64,
    unrealized_pnl: f64,
    realized_pnl: f64,
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
}

#[cfg(test)]
mod tests {
    use super::Order;
    use super::Portfolio;
    use super::Position;

    #[test]
    fn test_position() {
        let epsilon: f64 = 1e-9;
        let mut p1 = Position::new();
        p1.make_order(&Order::new(10.0, 1.0));
        assert!(
            (p1.market_value - 10.0).abs() < epsilon,
            "p1.market_value is not approximately 10.0"
        );
        p1.update_market_price(10.1);
        assert!(
            (p1.unrealized_pnl - 0.1).abs() < epsilon,
            "p1.unrealized_pnl is not approximately 0.1"
        );
        p1.update_market_price(11.0);
        assert!(
            (p1.unrealized_pnl - 1.0).abs() < epsilon,
            "p1.unrealized_pnl is not approximately 1.0"
        );
        p1.make_order(&Order::new(10.5, -2.0));
        assert!(
            (p1.realized_pnl - 0.5).abs() < epsilon,
            "p1.realized_pnl is not approximately 0.5"
        );
        p1.update_market_price(10.1);
        assert!(
            (p1.unrealized_pnl - 0.4).abs() < epsilon,
            "p1.unrealized_pnl is not approximately 0.4"
        );
        println!("{:?}", p1);
        p1.make_order(&Order::new(9.9, 4.0));
        println!("{:?}", p1);

        assert!(
            (p1.realized_pnl - 1.1).abs() < epsilon,
            "p1.realized_pnl is not approximately 0.5"
        );
    }

    #[test]
    fn test_portfolio() {
        let mut port = Portfolio::new(
            100.0,
            20.0,
            vec!["btcusdt".to_string(), "ethusdt".to_string()],
        );
        let orders = vec![
            ("btcusdt".to_string(), Order::new(68000.0, 0.01)),
            ("ethusdt".to_string(), Order::new(3400.0, 0.1)),
        ];
        port.make_orders(&orders.into_iter().collect()).unwrap();
        println!("{:#?}", port);
        let market_price = vec![
            ("btcusdt".to_string(), 69000.0),
            ("ethusdt".to_string(), 3450.0),
        ];
        port.update_market_price(market_price.into_iter().collect());
        println!("{:#?}", port);
        let orders = vec![
            ("btcusdt".to_string(), Order::new(68500.0, -0.02)),
            ("ethusdt".to_string(), Order::new(3500.0, -0.2)),
        ];
        port.make_orders(&orders.into_iter().collect()).unwrap();
        println!("{:#?}", port);

        let orders = vec![
            ("btcusdt".to_string(), Order::new(68000.0, 0.01)),
            ("ethusdt".to_string(), Order::new(3400.0, 0.1)),
        ];
        port.make_orders(&orders.into_iter().collect()).unwrap();
        println!("{:#?}", port);
    }
}
