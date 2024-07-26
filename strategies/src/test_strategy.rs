use public::base_enum::order_enums::OrderSide;
use public::base_model::market_model::kline_model::Kline;
use public::base_model::trade_model::order_model::Order;
use public::strategy_model::strategy_portfolio::StrategyPortfolio;
use public::tools::time_tools;
use tracing::info;
use trade_engine::BaseStrategy;

use quant_libs::tech_analysis::ma;
use std::collections::HashMap;

pub struct TestStrategy {
    strategy_name: String,
    symbol: String,
    sma_fast: ma::SMA,
    sma_slow: ma::SMA,
    ema_fast: ma::EMA,
    ema_slow: ma::EMA,
}

impl TestStrategy {
    pub fn new(
        strategy_name: String,
        symbol: String,
        sma_fast_period: usize,
        sma_slow_period: usize,
        ema_fast_period: usize,
        ema_slow_period: usize,
    ) -> Self {
        Self {
            strategy_name,
            symbol,
            sma_fast: ma::SMA::new(sma_slow_period),
            sma_slow: ma::SMA::new(sma_fast_period),
            ema_fast: ma::EMA::new(ema_fast_period),
            ema_slow: ma::EMA::new(ema_slow_period),
        }
    }
}

impl BaseStrategy for TestStrategy {
    fn on_schedule(
        &mut self,
        klines: &HashMap<String, Kline>,
        portfolio: &StrategyPortfolio,
    ) -> Option<HashMap<String, Order>> {
        let mut res = HashMap::new();
        let last_fast_ema = self.ema_fast.get();
        let last_slow_ema = self.ema_slow.get();
        let kline = klines.get(&self.symbol).unwrap();

        self.sma_fast.add(kline.get_close());
        self.sma_slow.add(kline.get_close());
        self.ema_fast.add(kline.get_close());
        self.ema_slow.add(kline.get_close());
        let cur_time = time_tools::get_datetime_from_timestamp(kline.get_open_time()).to_string();

        if let Some(current_position) = portfolio.get_position(&self.symbol) {
            let mut new_pos = 0.0;
            let mut new_side = current_position.get_side();
            if current_position.get_quantity() != 0.0
                && kline.get_close() < self.sma_slow.get()
                && current_position.get_side() == OrderSide::BUY
            {
                new_pos = current_position.get_quantity();
                new_side = OrderSide::SELL;
                let close_info = format!(
                    "datetime: {}, sma_slow: {}, close: {}, CLOSE LONG POSITION: {}, SELL!!!!",
                    cur_time,
                    self.sma_slow.get(),
                    kline.get_close(),
                    new_pos,
                );
                info!("{}", close_info);
            }

            if current_position.get_quantity() != 0.0
                && kline.get_close() > self.sma_slow.get()
                && current_position.get_side() == OrderSide::SELL
            {
                new_pos = current_position.get_quantity();
                new_side = OrderSide::BUY;
                let close_info = format!(
                    "datetime: {}, sma_slow: {}, close: {}, CLOSE SHORT POSITION: {}, BUY!!!!",
                    cur_time,
                    self.sma_slow.get(),
                    kline.get_close(),
                    new_pos,
                );
                info!("{}", close_info);
            }

            if current_position.get_quantity() == 0.0 {
                if self.sma_fast.get() > self.sma_slow.get()
                    && last_fast_ema < last_slow_ema
                    && self.ema_fast.get() > self.ema_slow.get()
                    && self.sma_slow.get() != 0.0
                {
                    let available_cash = portfolio.get_available_cash();
                    let msg = format!(
                        "datetime: {}, sma_slow: {}, sma_fast: {}, ema_fast: {}, ema_slow: {}, last_fast_ema: {}, last_slow_ema: {}, BUY!!!!",
                        cur_time,
                        self.sma_slow.get(),
                        self.sma_fast.get(),
                        self.ema_fast.get(),
                        self.ema_slow.get(),
                        last_fast_ema,
                        last_slow_ema
                    );
                    info!("{}", msg);
                    new_pos = available_cash * 0.9 / kline.get_close() * portfolio.get_leverage_rate();
                    new_side = OrderSide::BUY;
                } else if self.sma_fast.get() < self.sma_slow.get()
                    && last_fast_ema > last_slow_ema
                    && self.ema_fast.get() < self.ema_slow.get()
                    && self.ema_fast.get() != 0.0
                    && self.sma_fast.get() != 0.0
                {
                    let available_cash = portfolio.get_available_cash();
                    let msg = format!(
                        "datetime: {}, sma_slow: {}, sma_fast: {}, ema_fast: {}, ema_slow: {}, last_fast_ema: {}, last_slow_ema: {}, SELL!!!!",
                        cur_time,
                        self.sma_slow.get(),
                        self.sma_fast.get(),
                        self.ema_fast.get(),
                        self.ema_slow.get(),
                        last_fast_ema,
                        last_slow_ema
                    );
                    info!("{}", msg);
                    new_pos = available_cash * 0.9 / kline.get_close() * portfolio.get_leverage_rate();
                    new_side = OrderSide::SELL;
                }
            }

            if new_pos != 0.0 {
                let mut order = Order::default();
                order.set_price(kline.get_close());
                order.set_qty(new_pos);
                order.set_symbol(&self.symbol);
                order.set_side(new_side);
                res.insert(self.symbol.clone(), order);
            }
        }

        if res.is_empty() {
            None
        } else {
            for (s, order) in res.iter() {
                let msg = format!(
                    "datetime: {}, symbol: {}, side: {}, price: {}, qty: {}",
                    cur_time,
                    s,
                    order.get_side().string(),
                    order.get_price(),
                    order.get_qty()
                );
                info!("{}", msg);
            }
            Some(res)
        }
    }

    fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use base_libs::tools::time_tools;
    use trade_engine::StrategyEngine;
    
    #[tokio::test]
    async fn test_test_strategy() {
        tracing_subscriber::fmt::init();
        

        let strategy = TestStrategy::new(
            "test_strategy".to_string(),
            "btcusdt".to_string(),
            5,
            10,
            5,
            10,
        );
        let symbols = vec!["btcusdt".to_string()];
        let start_date = time_tools::get_datetime_from_str("2024-07-10 00:00:00");
        let start_timestamp = time_tools::get_timestamp_from_datetime(start_date);
        let mut trade_engine = StrategyEngine::new(
            symbols.clone(),
            StrategyPortfolio::new(10000.0, 50.0, symbols.clone()),
            HashMap::new(),
            start_timestamp,
            Box::new(strategy),
            true,
        );
        trade_engine.run().await;
        
    }
}