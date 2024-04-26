use base_libs::base_strategy::base_strategy::{BaseStrategy, TargetPosition};
use base_libs::base_strategy::portfolio;
use base_libs::market_data_module::general_data;
use base_libs::tools::time_tools;
use quant_libs::tech_analysis::ma;
use quant_libs::tech_analysis::rsi;
use quant_libs::tech_analysis::super_trend;
use std::collections::HashMap;
pub struct RmaStrategy {
    strategy_name: String,
    symbol: String,
    super_trend: super_trend::SuperTrend,
}

impl RmaStrategy {
    pub fn new(strategy_name: String, symbol: String, period: usize) -> Self {
        Self {
            strategy_name,
            symbol,
            super_trend: super_trend::SuperTrend::new(period, 3.0),
        }
    }
}

impl BaseStrategy for RmaStrategy {
    fn on_schedule(
        &mut self,
        klines: &HashMap<String, general_data::Kline>,
        portfolio: &portfolio::Portfolio,
    ) -> HashMap<String, TargetPosition> {
        let kline = klines.get(&self.symbol).unwrap();
        self.super_trend.add(kline.clone());
        let cur_time = time_tools::get_datetime_from_timestamp(kline.get_open_time()).to_string();
        println!(
            "{}: cur_up: {} cur_dn: {} cur_trend: {}",
            cur_time,
            self.super_trend.get().0,
            self.super_trend.get().1,
            self.super_trend.get().2,
        );
        let res = HashMap::new();
        res
    }
    fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }
}
