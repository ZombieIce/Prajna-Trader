use public::base_model::market_model::kline_model::Kline;
use public::base_model::trade_model::order_model::Order;
use public::strategy_model::strategy_portfolio::StrategyPortfolio;
use services::mongo_engine::MongoEngine;
use std::collections::HashMap;

pub trait BaseStrategy {
    fn on_schedule(
        &mut self,
        klines: &HashMap<String, Kline>,
        portfolio: &StrategyPortfolio,
    ) -> Option<HashMap<String, Order>>;

    fn get_strategy_name(&self) -> String {
        "BaseStrategy".to_string()
    }
}

pub struct StrategyEngine {
    symbols: Vec<String>,
    mongo_client: MongoEngine,
    portfolio: StrategyPortfolio,
    kline_data: HashMap<String, Vec<Kline>>,
    start_date: i64,
    strategy: Box<dyn BaseStrategy>,
    is_backtest: bool,
}

impl StrategyEngine {
    pub fn new(
        symbols: Vec<String>,
        portfolio: StrategyPortfolio,
        kline_data: HashMap<String, Vec<Kline>>,
        start_date: i64,
        strategy: Box<dyn BaseStrategy>,
        is_backtest: bool,
    ) -> Self {
        let mongo_client = MongoEngine::default();
        StrategyEngine {
            symbols,
            mongo_client,
            portfolio,
            kline_data,
            start_date,
            strategy,
            is_backtest,
        }
    }

    async fn prepare_data(&mut self) {
        self.load_symbol_infos().await;
        self.load_history_klines().await;
    }

    async fn load_symbol_infos(&mut self) {
        match self.mongo_client.get_exchange_info().await {
            Some(exchange_info) => {
                let symbol_infos = exchange_info.get_symbol_info_map(&self.symbols);
                self.portfolio.set_symbol_infos(symbol_infos);
                tracing::info!("Get symbol info from exchange info");
            }
            None => {
                tracing::error!("Failed to get exchange info");
            }
        }
    }

    async fn load_history_klines(&mut self) {
        for symbol in &self.symbols {
            match self
                .mongo_client
                .fetch_klines(symbol, self.start_date)
                .await
            {
                Ok(stored_klines) => match stored_klines {
                    Some(klines) => {
                        self.kline_data.insert(symbol.clone(), klines);
                        tracing::info!("Get klines for symbol: {}", symbol);
                    }
                    None => {
                        tracing::info!("No klines for symbol: {}", symbol);
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to get klines for symbol: {}, error: {}", symbol, e);
                }
            }
        }
    }

    fn format_his_klines(&self) -> Vec<HashMap<String, Kline>> {
        let mut res: Vec<HashMap<String, Kline>> = Vec::new();
        let len = self.kline_data.get(&self.symbols[0]).unwrap().len();
        for i in 0..len {
            let mut klines: HashMap<String, Kline> = HashMap::new();
            for symbol in &self.symbols {
                klines.insert(
                    symbol.clone(),
                    self.kline_data.get(symbol).unwrap()[i].clone(),
                );
            }
            res.push(klines);
        }
        res
    }

    pub async fn run(&mut self) {
        self.prepare_data().await;
        if self.is_backtest {
            self.back_test();
        }
    }

    fn format_order(&self, orders: HashMap<String, Order>) -> HashMap<String, Order> {
        let mut res: HashMap<String, Order> = HashMap::new();
        for (s, order) in orders {
            let symbol_info = self.portfolio.get_symbol_infos().get(&s).unwrap();
            let price_prec = symbol_info.get_price_precision();
            let qty_prec = symbol_info.get_quantity_precision();
            let mut order = order.clone();
            order.format_order(price_prec, qty_prec);
            res.insert(s, order);
        }
        res
    }

    fn back_test(&mut self) {
        let format_klines = self.format_his_klines();
        let mut tmp_orders: HashMap<String, Order> = HashMap::new();
        for klines in format_klines {
            if !tmp_orders.is_empty() {
                for (s, order) in tmp_orders.iter_mut() {
                    let cur_kline = klines.get(s).unwrap();
                    order.set_filled_qty(order.get_qty());
                    order.set_avg_price(cur_kline.get_open());
                }
                match self.portfolio.make_back_test_order(tmp_orders.clone()) {
                    Ok(_) => tmp_orders.clear(),
                    Err(e) => {
                        tracing::error!("Failed to make back test order: {}", e);
                        break;
                    }
                }
            }

            if let Some(orders) = self.strategy.on_schedule(&klines, &self.portfolio) {
                tmp_orders = self.format_order(orders);
            }
            self.portfolio.update_back_test_market_price(&klines);
            match self.portfolio.update_back_test_value() {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Failed to update back test value: {}", e);
                    break;
                }
            }
            self.portfolio
                .update_pnl_records(klines.get(&self.symbols[0]).unwrap().get_open_time())
        }
        self.portfolio.show_summary();
    }
}
