use libs::base_strategy::base_strategy::{BaseStrategy, TargetPosition};
use libs::base_strategy::portfolio;
use libs::market_data_module::general_data;
use libs::tools::time_tools;
use quant_libs::tech_analysis::ma;
use quant_libs::tech_analysis::rsi;
use std::collections::HashMap;
pub struct RmaStrategy {
    strategy_name: String,
    symbol: String,
    rsi: rsi::RSI,
}

impl RmaStrategy {
    pub fn new(strategy_name: String, symbol: String, period: usize) -> Self {
        Self {
            strategy_name,
            symbol,
            rsi: rsi::RSI::new(period),
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
        self.rsi.add(kline.get_close());

        let cur_time = time_tools::get_datetime_from_timestamp(kline.get_open_time()).to_string();
        println!("Time: {}, rsi: {}", cur_time, self.rsi.get());
        let res = HashMap::new();
        res
    }
}
