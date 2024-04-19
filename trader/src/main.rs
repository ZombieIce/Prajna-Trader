use chrono::{DateTime, FixedOffset};
use chrono::{TimeZone, Utc};
use libs::tools::time_tools;
use libs::{
    base_strategy::{
        base_strategy::{BaseStrategy, StrategyContext, TargetPosition},
        portfolio,
    },
    market_data_module::{general_data, general_enum},
};
use quant_libs::tech_analysis::ma;
use services::market_data_engine::market_data_engine::MarketDataEngine;
use std::collections::HashMap;
use std::fmt::format;
use tracing::info;
use tracing_subscriber;

struct TestStrategy {
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
    ) -> HashMap<String, TargetPosition> {
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
        res
    }

    fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // tracing_subscriber::fmt::init();
    // info!("start!");
    // let symbols = vec![
    //     "solusdt".to_string(),
    //     "avaxusdt".to_string(),
    //     "btcusdt".to_string(),
    //     "ethusdt".to_string(),
    //     "dogeusdt".to_string(),
    //     "bnbusdt".to_string(),
    //     "adausdt".to_string(),
    //     "tonusdt".to_string(),
    //     "1000shibusdt".to_string(),
    //     "dotusdt".to_string(),
    //     "linkusdt".to_string(),
    //     "trxusdt".to_string(),
    //     "maticusdt".to_string(),
    //     "nearusdt".to_string(),
    //     "uniusdt".to_string(),
    // ];

    // let mut m_engine = MarketDataEngine::default();
    // m_engine.subscribe_symbols(&symbols);
    // m_engine.start().await;
    let symbol = "btcusdt".to_string();
    let start_date_timestamp =
        time_tools::get_datetime_from_str("2023-01-01 00:00:00").timestamp_millis();

    let test_strategy =
        TestStrategy::new("TestStrategy".to_string(), symbol.clone(), 33, 88, 8, 13);
    let mut strategy = StrategyContext::new(
        vec![symbol.clone()],
        1000.0,
        start_date_timestamp,
        general_enum::Interval::Min15,
        20.0,
        true,
        Box::new(test_strategy),
    );
    strategy.start().await;
}
