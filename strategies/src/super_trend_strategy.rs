use base_libs::base_strategy::base_strategy::BaseStrategy;
use base_libs::base_strategy::common_module::TargetPosition;
use base_libs::base_strategy::portfolio;
use base_libs::market_data_module::{general_data, general_enum};
use base_libs::tools::time_tools;
use quant_libs::tech_analysis::super_trend;
use std::collections::HashMap;
pub struct SuperTrendStrategy {
    strategy_name: String,
    symbol: String,
    super_trend: super_trend::SuperTrend,
    min15_kline: general_data::CombineKline,
}

impl SuperTrendStrategy {
    pub fn new(strategy_name: String, symbol: String, period: usize) -> Self {
        Self {
            strategy_name,
            symbol,
            super_trend: super_trend::SuperTrend::new(period, 3.0),
            min15_kline: general_data::CombineKline::new(vec![], general_enum::Interval::Min15),
        }
    }
}

impl BaseStrategy for SuperTrendStrategy {
    fn on_schedule(
        &mut self,
        klines: &HashMap<String, general_data::Kline>,
        _portfolio: &portfolio::Portfolio,
    ) -> Option<HashMap<String, TargetPosition>> {
        let kline = klines.get(&self.symbol).unwrap();
        self.min15_kline.add(kline.clone());
        if let Some(min15_kline) = self.min15_kline.get_kline() {
            self.super_trend.add(min15_kline);
            let cur_time =
                time_tools::get_datetime_from_timestamp(min15_kline.get_open_time()).to_string();
            println!(
                "{}: cur_up: {} cur_dn: {} cur_trend: {}",
                cur_time,
                self.super_trend.get().0,
                self.super_trend.get().1,
                self.super_trend.get().2,
            );
        }
        None
    }
    fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }
}
