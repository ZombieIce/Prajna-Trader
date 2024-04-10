use std::collections::HashMap;

use super::portfolio::Order;
use super::portfolio::Portfolio;
use crate::kline_basic;
use crate::market_data_module::general_data;
use crate::market_data_module::general_data::Kline;
use crate::market_data_module::general_enum;

#[derive(Debug)]
pub struct TargetPosition(f64);
impl TargetPosition {
    pub fn new(qty: f64) -> Self {
        Self(qty)
    }
}

pub trait BaseStrategy {
    fn on_schedule(
        &mut self,
        klines: &HashMap<String, general_data::Kline>,
        portfolio: &Portfolio,
    ) -> HashMap<String, TargetPosition>;

    fn get_strategy_name(&self) -> String {
        "BaseStrategy".to_string()
    }
}

pub struct StrategyContext {
    symbols: Vec<String>,
    portfolio: Portfolio,
    kline_data: HashMap<String, Vec<general_data::Kline>>,
    start_date: i64,
    interval: general_enum::Interval,
    is_back_test: bool,
    strategy: Box<dyn BaseStrategy>,
}

impl StrategyContext {
    pub fn new(
        symbols: Vec<String>,
        cash: f64,
        start_date: i64,
        interval: general_enum::Interval,
        leverage_rate: f64,
        is_back_test: bool,
        strategy: Box<dyn BaseStrategy>,
    ) -> Self {
        Self {
            symbols: symbols.clone(),
            portfolio: Portfolio::new(cash, leverage_rate, symbols.clone()),
            kline_data: HashMap::new(),
            start_date,
            interval,
            is_back_test,
            strategy,
        }
    }

    async fn init_kline(&mut self) {
        let mut max_start_date: i64 = 0;
        let mut tmp_kline_data: HashMap<String, Vec<general_data::Kline>> = HashMap::new();
        for symbol in &self.symbols {
            match kline_basic::fetch_klines(symbol, self.start_date, &self.interval).await {
                Some(klines) => {
                    if klines[0].get_open_time() > max_start_date {
                        max_start_date = klines[0].get_open_time();
                    }
                    tmp_kline_data.insert(symbol.clone(), klines);
                }
                None => {
                    println!("error: fetch klines {symbol} failed");
                }
            }
        }

        for symbol in &self.symbols {
            let klines = tmp_kline_data.get(symbol).unwrap();
            let klines: Vec<Kline> = klines
                .iter()
                .filter(|kline| kline.get_open_time() >= max_start_date)
                .cloned()
                .collect();
            self.kline_data.insert(symbol.clone(), klines);
        }
    }

    fn format_his_klines(&self) -> Vec<HashMap<String, general_data::Kline>> {
        let mut res: Vec<HashMap<String, general_data::Kline>> = vec![];
        let len = self.kline_data.get(&self.symbols[0]).unwrap().len();
        for i in 0..len {
            let mut kline_map: HashMap<String, general_data::Kline> = HashMap::new();
            for symbol in &self.symbols {
                let kline = self.kline_data.get(symbol).unwrap()[i].clone();
                kline_map.insert(symbol.clone(), kline);
            }
            res.push(kline_map);
        }
        res
    }

    fn format_his_price(&self, klines: &HashMap<String, Kline>) -> HashMap<String, f64> {
        let mut res: HashMap<String, f64> = HashMap::new();
        for (symbol, kline) in klines.iter() {
            res.insert(symbol.clone(), kline.get_close());
        }
        res
    }

    pub async fn start(&mut self) {
        self.init_kline().await;
        if self.is_back_test {
            self.back_test();
        } else {
            self.real_trade().await;
        }
    }

    fn convert_orders(
        &self,
        orders: &HashMap<String, TargetPosition>,
        klines: &HashMap<String, Kline>,
    ) -> HashMap<String, Order> {
        let mut res: HashMap<String, Order> = HashMap::new();
        for (symbol, target_position) in orders.iter() {
            let target_position = target_position.0;
            let price = klines.get(symbol).unwrap().get_open();

            if let Some(current_position) = self.portfolio.get_position(&symbol) {
                let order = Order::new(price, target_position - current_position.get_qty());
                res.insert(symbol.clone(), order);
            } else {
                let order = Order::new(price, target_position);
                res.insert(symbol.clone(), order);
            }
        }
        res
    }

    pub fn back_test(&mut self) {
        let format_klines = self.format_his_klines();
        let mut tmp_orders = HashMap::new();
        for klines in format_klines {
            // check if there are orders to execute
            // if there are, execute them
            // portfolio make orders
            if tmp_orders.len() > 0 {
                match self.portfolio.make_orders(&tmp_orders) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }

            // run strategy on schedule
            let orders = self.strategy.on_schedule(&klines, &self.portfolio);
            // if threre are orders to execute, convert them and store them in tmp_orders
            if orders.len() > 0 {
                tmp_orders = self.convert_orders(&orders, &klines);
            } else {
                tmp_orders.clear();
            }
            // else empty tmp_orders

            // portfolio update market price
            self.portfolio
                .update_market_price(self.format_his_price(&klines));
        }
    }

    pub async fn real_trade(&self) {
        // todo!("real trade");
    }

    pub fn get_kline(&self, symbol: &str) -> Option<&Vec<general_data::Kline>> {
        self.kline_data.get(symbol)
    }
}
