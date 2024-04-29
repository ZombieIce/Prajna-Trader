use base_libs::base_strategy::base_strategy::BaseStrategy;
use base_libs::base_strategy::common_module::TargetPosition;
use base_libs::base_strategy::portfolio;
use base_libs::market_data_module::general_data;
use base_libs::tools::time_tools;
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
        klines: &HashMap<String, general_data::Kline>,
        portfolio: &portfolio::Portfolio,
    ) -> Option<HashMap<String, TargetPosition>> {
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
            let mut new_pos = current_position.get_qty();
            if (current_position.get_qty() > 0.0 && kline.get_close() < self.sma_slow.get())
                || (current_position.get_qty() < 0.0 && kline.get_close() > self.sma_slow.get())
            {
                new_pos = 0.0;
                let close_info = format!(
                    "datetime: {}, sma_slow: {}, close: {}",
                    cur_time,
                    self.sma_slow.get(),
                    kline.get_close()
                );

                if current_position.get_qty() > 0.0 {
                    println!(
                        "{} CLOSE LONG POSITION: {}, SELL!!!!",
                        close_info,
                        current_position.get_qty()
                    );
                } else {
                    println!(
                        "{} CLOSE SHORT POSITION: {}, BUY!!!!",
                        close_info,
                        current_position.get_qty()
                    );
                }
            }
            if current_position.get_qty() == 0.0 {
                if self.sma_fast.get() > self.sma_slow.get()
                    && last_fast_ema < last_slow_ema
                    && self.ema_fast.get() > self.ema_slow.get()
                    && self.sma_slow.get() != 0.0
                {
                    let available_cash = portfolio.get_available_cash();
                    println!(
                        "datetime: {}, sma_slow: {}, sma_fast: {}, ema_fast: {}, ema_slow: {}, last_fast_ema: {}, last_slow_ema: {}, BUY!!!!",
                        cur_time,
                        self.sma_slow.get(),
                        self.sma_fast.get(),
                        self.ema_fast.get(),
                        self.ema_slow.get(),
                        last_fast_ema,
                        last_slow_ema
                    );
                    new_pos += available_cash / kline.get_close();
                } else if self.sma_fast.get() < self.sma_slow.get()
                    && last_fast_ema > last_slow_ema
                    && self.ema_fast.get() < self.ema_slow.get()
                    && self.ema_fast.get() != 0.0
                    && self.sma_fast.get() != 0.0
                {
                    let available_cash = portfolio.get_available_cash();
                    println!(
                        "datetime: {}, sma_slow: {}, sma_fast: {}, ema_fast: {}, ema_slow: {}, last_fast_ema: {}, last_slow_ema: {}, SELL!!!!",
                        cur_time,
                        self.sma_slow.get(),
                        self.sma_fast.get(),
                        self.ema_fast.get(),
                        self.ema_slow.get(),
                        last_fast_ema,
                        last_slow_ema
                    );
                    new_pos -= available_cash / kline.get_close();
                }
            }

            if new_pos != current_position.get_qty() {
                res.insert(self.symbol.clone(), TargetPosition::new(new_pos));
            }
        }
        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }
}
