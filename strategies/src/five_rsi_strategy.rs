use libs::base_strategy::base_strategy::{BaseStrategy, TargetPosition};
use libs::base_strategy::portfolio;
use libs::market_data_module::general_data;
use libs::tools::time_tools;
use quant_libs::tech_analysis::ma;
use quant_libs::tech_analysis::rsi;
use std::collections::HashMap;
pub struct FiveRsiStrategy {
    strategy_name: String,
    symbol: String,
    rsi: rsi::RSI,
    rsi_ma: ma::SMA,
    rsi_ma_vec: Vec<f64>,
    high_ema: ma::EMA,
    close_ema: ma::EMA,
    low_ema: ma::EMA,
    last_kline: Option<general_data::Kline>,
    last_bound: f64,
}

impl FiveRsiStrategy {
    pub fn new(
        strategy_name: String,
        symbol: String,
        rsi_period: usize,
        rsi_smooth_period: usize,
    ) -> Self {
        Self {
            strategy_name,
            symbol,
            rsi: rsi::RSI::new(rsi_period),
            rsi_ma: ma::SMA::new(rsi_smooth_period),
            rsi_ma_vec: Vec::new(),
            high_ema: ma::EMA::new(200),
            close_ema: ma::EMA::new(200),
            low_ema: ma::EMA::new(200),
            last_kline: None,
            last_bound: 0.0,
        }
    }
}

impl BaseStrategy for FiveRsiStrategy {
    fn on_schedule(
        &mut self,
        klines: &HashMap<String, general_data::Kline>,
        portfolio: &portfolio::Portfolio,
    ) -> HashMap<String, TargetPosition> {
        let kline = klines.get(&self.symbol).unwrap();
        let cur_time = time_tools::get_datetime_from_timestamp(kline.get_open_time()).to_string();
        self.rsi.add(kline.get_close());
        self.rsi_ma.add(self.rsi.get());
        self.high_ema.add(kline.get_high());
        self.close_ema.add(kline.get_close());
        self.low_ema.add(kline.get_low());
        self.rsi_ma_vec.push(self.rsi_ma.get());
        if self.rsi_ma_vec.len() > 2 {
            self.rsi_ma_vec.remove(0);
        }
        let mut res = HashMap::new();

        if let Some(current_position) = portfolio.get_position(&self.symbol) {
            let mut new_pos = current_position.get_qty();
            if current_position.get_qty() > 0.0
                && (self.rsi_ma.get() > 70.0
                    || kline.get_close() < self.close_ema.get()
                    || kline.get_close() < self.last_bound)
            {
                new_pos = 0.0;
                println!(
                    "Time: {}, Close: {}, rsi_ma: {}, CLOSE LONG pos: {}",
                    cur_time,
                    kline.get_close(),
                    self.rsi_ma.get(),
                    current_position.get_qty()
                );
            }

            if current_position.get_qty() < 0.0
                && (self.rsi_ma.get() < 30.0
                    || kline.get_close() > self.close_ema.get()
                    || kline.get_close() > self.last_bound)
            {
                new_pos = 0.0;
                println!(
                    "Time: {}, Close: {}, rsi_ma: {}, CLOSE SHORT",
                    cur_time,
                    kline.get_close(),
                    self.rsi_ma.get()
                );
            }

            if current_position.get_qty() == 0.0 && self.rsi_ma_vec.len() == 2 {
                if kline.get_close() >= self.close_ema.get()
                    && self.rsi_ma_vec[1] < 25.0
                    && self.rsi_ma_vec[1] > self.rsi_ma_vec[0]
                {
                    let available_cash = portfolio.get_available_cash();
                    new_pos = available_cash * 0.3 / kline.get_close();
                    self.last_bound = kline.get_low();
                    println!(
                        "Time: {}, Close: {}, rsi_ma: {}, OPEN LONG pos: {}",
                        cur_time,
                        kline.get_close(),
                        self.rsi_ma.get(),
                        new_pos
                    )
                }

                if kline.get_close() <= self.close_ema.get()
                    && self.rsi_ma_vec[1] > 75.0
                    && self.rsi_ma_vec[1] < self.rsi_ma_vec[0]
                {
                    let available_cash = portfolio.get_available_cash();
                    new_pos = -available_cash * 0.3 / kline.get_close();
                    self.last_bound = kline.get_high();
                    println!(
                        "Time: {}, Close: {}, rsi_ma: {}, OPEN SHORT pos: {}",
                        cur_time,
                        kline.get_close(),
                        self.rsi_ma.get(),
                        new_pos
                    )
                }
            }

            if new_pos != current_position.get_qty() {
                res.insert(self.symbol.clone(), TargetPosition::new(new_pos));
            }
        }
        self.last_kline = Some(kline.clone());
        res
    }
}
