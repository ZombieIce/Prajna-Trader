use std::collections::HashMap;

use super::common_module::TargetPosition;
use super::order::Order;
use super::order::OrderParse;
use super::portfolio::Portfolio;
use crate::kline_basic;
use crate::market_data_module::general_data;
use crate::market_data_module::general_data::Kline;
use crate::market_data_module::general_enum;
use crate::mongo_engine::MongoEngine;
use crate::tools::time_tools;
use chrono::Duration;
use plotters::prelude::*;
use tracing;
use tracing::error;

#[derive(Debug, Clone, Copy)]
pub struct PnlRecord {
    timestamp: i64,
    pnl: f64,
    net_value: f64,
}

impl PnlRecord {
    pub fn new(timestamp: i64, pnl: f64, net_value: f64) -> Self {
        Self {
            timestamp,
            pnl,
            net_value,
        }
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_pnl(&self) -> f64 {
        self.pnl
    }

    pub fn get_net_value(&self) -> f64 {
        self.net_value
    }
}

pub trait BaseStrategy {
    fn on_schedule(
        &mut self,
        klines: &HashMap<String, general_data::Kline>,
        portfolio: &Portfolio,
    ) -> Option<HashMap<String, TargetPosition>>;

    fn get_strategy_name(&self) -> String {
        "BaseStrategy".to_string()
    }
}

pub struct StrategyContext {
    symbols: Vec<String>,
    portfolio: Portfolio,
    kline_data: HashMap<String, Vec<general_data::Kline>>,
    start_date: i64,
    strategy: Box<dyn BaseStrategy>,
    pnl_records: Vec<PnlRecord>,
    symbol_infos: HashMap<String, general_data::SymbolInfo>,
    real_kline_data: HashMap<String, general_data::Kline>,
    order_parser: OrderParse,
}
unsafe impl Send for StrategyContext {}

impl StrategyContext {
    pub fn new(
        symbols: Vec<String>,
        cash: f64,
        start_date: i64,
        leverage_rate: f64,
        strategy: Box<dyn BaseStrategy>,
    ) -> Self {
        Self {
            symbols: symbols.clone(),
            portfolio: Portfolio::new(cash, leverage_rate, symbols.clone()),
            kline_data: HashMap::new(),
            start_date,
            strategy,
            pnl_records: vec![],
            symbol_infos: HashMap::new(),
            real_kline_data: HashMap::new(),
            order_parser: OrderParse::new(HashMap::new()),
        }
    }

    async fn init_symbol_info(&mut self) {
        let mongo_engine = MongoEngine::default();
        match mongo_engine.get_exchange_info().await {
            Some(exchange_info) => {
                self.symbol_infos = exchange_info.get_symbol_info_map(&self.symbols);
            }
            None => {
                error!("fetch exchange info failed");
            }
        }
    }

    async fn init_pure_historical_data(&mut self) {
        self.init_symbol_info().await;
        self.order_parser
            .set_symbol_infos(self.symbol_infos.clone());
        let mut max_start_date: i64 = 0;
        let mut tmp_kline_data: HashMap<String, Vec<general_data::Kline>> = HashMap::new();
        for symbol in &self.symbols {
            match kline_basic::fetch_klines(symbol, self.start_date, &general_enum::Interval::Min5)
                .await
            {
                Some(klines) => {
                    if klines.len() == 0 {
                        error!("error: fetch kliens {symbol} failed");
                        return;
                    }
                    if klines[0].get_open_time() > max_start_date {
                        max_start_date = klines[0].get_open_time();
                    }
                    tmp_kline_data.insert(symbol.clone(), klines);
                }
                None => {
                    error!("error: fetch kliens {symbol} failed");
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

    fn get_format_close(&self, klines: &HashMap<String, Kline>) -> HashMap<String, f64> {
        let mut res: HashMap<String, f64> = HashMap::new();
        for (symbol, kline) in klines.iter() {
            res.insert(symbol.clone(), kline.get_close());
        }
        res
    }

    pub async fn start_backtest(&mut self) {
        self.init_pure_historical_data().await;
        self.back_test();
    }

    pub fn back_test(&mut self) {
        let format_klines = self.format_his_klines();
        let mut tmp_orders: HashMap<String, TargetPosition> = HashMap::new();

        for klines in format_klines {
            if tmp_orders.len() != 0 {
                match self.order_parser.convert_backetst_order(
                    &tmp_orders,
                    &klines,
                    &self.portfolio,
                ) {
                    Ok(orders) => {
                        let mut orders = orders;
                        match self.portfolio.make_orders(&mut orders) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("error: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("error: {}", e);
                        break;
                    }
                }
            }

            if let Some(orders) = self.strategy.on_schedule(&klines, &self.portfolio) {
                tmp_orders = orders;
            } else {
                tmp_orders.clear();
            }
            self.portfolio
                .update_market_price(self.get_format_close(&klines));
            self.pnl_records.push(PnlRecord::new(
                klines.get(&self.symbols[0]).unwrap().get_open_time(),
                self.portfolio.get_pnl(),
                self.portfolio.get_total_value() / self.portfolio.get_starting_cash(),
            ));
        }
        self.back_test_summary();
    }

    fn back_test_plot(&self, pnl_vec: Vec<f64>) -> Result<(), Box<dyn std::error::Error>> {
        let title = format!(
            "{} {}",
            self.strategy.get_strategy_name(),
            time_tools::get_now_string()
        );
        let file_name = format!("{}.png", title);
        let root_area = BitMapBackend::new(&file_name, (1024, 768)).into_drawing_area();
        root_area.fill(&WHITE)?;
        let (upper, lower) = root_area.split_vertically(384);

        let net_value_data: Vec<(i64, f64)> = self
            .pnl_records
            .iter()
            .map(|record| (record.get_timestamp(), record.get_net_value()))
            .collect();

        let mut max_drawdown_data: Vec<(i64, f64)> = Vec::new();
        let mut max_value = net_value_data[0].1;
        for (timestamp, net_value) in net_value_data.iter() {
            if *net_value > max_value {
                max_value = *net_value;
            }
            let drawdown = (*net_value - max_value) / max_value;
            max_drawdown_data.push((*timestamp, drawdown));
        }

        let max_value =
            net_value_data.iter().fold(
                1.0,
                |acc, (_, net_value)| {
                    if *net_value > acc {
                        *net_value
                    } else {
                        acc
                    }
                },
            ) + 0.01;
        let min_value =
            net_value_data.iter().fold(
                1.0,
                |acc, (_, net_value)| {
                    if *net_value < acc {
                        *net_value
                    } else {
                        acc
                    }
                },
            ) - 0.01;

        let (start_time, end_time) = (
            time_tools::get_datetime_from_timestamp(net_value_data[0].0) - Duration::minutes(30),
            time_tools::get_datetime_from_timestamp(net_value_data[net_value_data.len() - 1].0)
                + Duration::minutes(30),
        );
        let max_drawdown =
            max_drawdown_data.iter().fold(
                0.0,
                |acc, (_, drawdown)| {
                    if *drawdown < acc {
                        *drawdown
                    } else {
                        acc
                    }
                },
            ) - 0.01;

        let mut uppper_chart = ChartBuilder::on(&upper)
            .x_label_area_size(5)
            .y_label_area_size(20)
            .right_y_label_area_size(20)
            .margin(20)
            .caption(&title, ("sans-serif", 20))
            .build_cartesian_2d(start_time..end_time, min_value..max_value)?
            .set_secondary_coord(start_time..end_time, max_drawdown..0.0);

        uppper_chart
            .configure_mesh()
            .x_labels(5)
            .y_labels(10)
            .y_label_formatter(&|v| format!("{:.2}", v))
            .y_desc("Net Value")
            .disable_mesh()
            .draw()?;

        uppper_chart
            .configure_secondary_axes()
            .y_desc("Max Drawdown")
            .draw()?;
        uppper_chart.draw_series(LineSeries::new(
            net_value_data.iter().map(|(timestamp, net_value)| {
                (
                    time_tools::get_datetime_from_timestamp(*timestamp),
                    *net_value,
                )
            }),
            &BLUE,
        ))?;
        uppper_chart.draw_secondary_series(AreaSeries::new(
            max_drawdown_data.iter().map(|(timestamp, drawdown)| {
                (
                    time_tools::get_datetime_from_timestamp(*timestamp),
                    *drawdown,
                )
            }),
            0.0,
            &RED.mix(0.3),
        ))?;

        let format_pnl: Vec<i64> = pnl_vec.iter().map(|pnl| pnl.round() as i64).collect();
        let (lower_bound, upper_bound) = (
            format_pnl
                .iter()
                .fold(0, |acc, pnl| if *pnl < acc { *pnl } else { acc })
                - 1,
            format_pnl
                .iter()
                .fold(0, |acc, pnl| if *pnl > acc { *pnl } else { acc })
                + 1,
        );
        let mut lower_chart = ChartBuilder::on(&lower)
            .x_label_area_size(5)
            .y_label_area_size(20)
            .right_y_label_area_size(20)
            .margin(20)
            .caption("order pnl histogram", ("sans-serif", 20))
            .build_cartesian_2d(lower_bound..upper_bound, 0..format_pnl.len() as i64)?;
        lower_chart
            .configure_mesh()
            .x_labels(5)
            .y_labels(10)
            .x_desc("Pnl")
            .y_desc("Count")
            .disable_mesh()
            .draw()?;
        lower_chart.draw_series(
            Histogram::vertical(&lower_chart)
                .style(RED.mix(0.5).filled())
                .data(format_pnl.iter().map(|pnl| (*pnl, 1))),
        )?;

        root_area.present()?;

        Ok(())
    }

    fn back_test_statistic(&self) {
        for s in self.symbols.iter() {
            if let Some(orders) = self.portfolio.get_orders(&s) {
                if orders.len() <= 1 {
                    continue;
                }
                println!("symbol: {}", s);
                let pnl_vec = self.win_rate_statistic(orders);
                match self.back_test_plot(pnl_vec) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }
        }
    }

    fn ratio_statistic(&self) {
        let mut returns: Vec<f64> = Vec::new();
        for idx in 1..self.pnl_records.len() {
            let pnl = (self.pnl_records[idx].get_net_value()
                - self.pnl_records[idx - 1].get_net_value())
                / self.pnl_records[idx - 1].get_net_value();
            returns.push(pnl);
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let mut squared_diff_sum = 0.0;
        for ret in &returns {
            squared_diff_sum += (ret - mean_return).powi(2);
        }
        let std_dev = (squared_diff_sum / returns.len() as f64).sqrt();
        let risk_free_rate = 0.0; // Set the risk-free rate here
        let sharpe_ratio = (mean_return - risk_free_rate) / std_dev;

        let downside_returns: Vec<f64> = returns
            .iter()
            .filter(|&ret| *ret < risk_free_rate)
            .map(|&ret| ret)
            .collect();
        let mean_downside_return =
            downside_returns.iter().sum::<f64>() / downside_returns.len() as f64;
        let mut squared_downside_diff_sum = 0.0;
        for ret in &downside_returns {
            squared_downside_diff_sum += (ret - mean_downside_return).powi(2);
        }
        let downside_std_dev = (squared_downside_diff_sum / downside_returns.len() as f64).sqrt();
        let sortino_ratio = (mean_return - risk_free_rate) / downside_std_dev;
        let sqrt_ratio = 288.0_f64.sqrt();

        println!("Sharpe Ratio: {}", sharpe_ratio * sqrt_ratio);
        println!("Sortino Ratio: {}", sortino_ratio * sqrt_ratio);
    }

    fn win_rate_statistic(&self, orders: &Vec<Order>) -> Vec<f64> {
        let mut win_count = 0;
        let mut total_count = 0;
        let mut max_win = 0.0;
        let mut max_loss = 0.0;
        let mut win_pnl_arr: Vec<f64> = Vec::new();
        let mut loss_pnl_arr: Vec<f64> = Vec::new();
        let mut holding_period_arr: Vec<i64> = Vec::new();
        let mut max_hold_period = 0;
        let mut min_hold_period = 1000000000;
        let mut pnl_vec: Vec<f64> = Vec::new();

        let mut accum_qty = 0.0;
        let mut tmp_orders: Vec<Order> = Vec::new();
        for ord in orders.iter() {
            tmp_orders.push(ord.clone());
            accum_qty += ord.get_qty();
            if accum_qty == 0.0 {
                let cur_pnl = tmp_orders.iter().fold(0.0, |acc, ord| {
                    acc - ord.get_price() * ord.get_qty() - ord.get_fee()
                });
                pnl_vec.push(cur_pnl);
                let cur_holding_period = tmp_orders[tmp_orders.len() - 1].get_timestamp()
                    - tmp_orders[0].get_timestamp();
                if cur_holding_period > max_hold_period {
                    max_hold_period = cur_holding_period;
                }
                if cur_holding_period < min_hold_period {
                    min_hold_period = cur_holding_period;
                }
                holding_period_arr.push(cur_holding_period);
                if cur_pnl > 0.0 {
                    win_count += 1;
                    win_pnl_arr.push(cur_pnl);
                } else {
                    loss_pnl_arr.push(cur_pnl);
                }
                if cur_pnl > max_win {
                    max_win = cur_pnl;
                }

                if cur_pnl < max_loss {
                    max_loss = cur_pnl;
                }

                total_count += 1;
                tmp_orders.clear();
            }
        }

        let avg_win_pnl = win_pnl_arr.iter().sum::<f64>() / win_pnl_arr.len() as f64;
        let avg_loss_pnl = loss_pnl_arr.iter().sum::<f64>() / loss_pnl_arr.len() as f64;
        let avg_holding_period =
            holding_period_arr.iter().sum::<i64>() / holding_period_arr.len() as i64;

        fn format_holding_period(holding_period: i64) -> String {
            let duration = chrono::Duration::seconds(holding_period / 1000);
            let hours = duration.num_hours();
            let minutes = duration.num_minutes() % 60;
            let seconds = duration.num_seconds() % 60;
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        }

        println!("trades_count: {}, win rate: {}, max_holding_period: {}, min_holding_period: {}, average_holding_period: {}",
                 total_count, win_count as f64 / total_count as f64, format_holding_period(max_hold_period), format_holding_period(min_hold_period), format_holding_period(avg_holding_period));

        println!(
            "win_loss_pnl_rate: {}, average_win_pnl: {}, average_loss_pnl: {}, average_win_loss_rate: {},  max_win: {}, max_loss: {}",
            (win_pnl_arr.iter().sum::<f64>() / loss_pnl_arr.iter().sum::<f64>()).abs(),
            avg_win_pnl,
            avg_loss_pnl,
            (avg_win_pnl / avg_loss_pnl).abs(),
            max_win,
            max_loss
        );
        pnl_vec
    }

    fn back_test_summary(&self) {
        self.back_test_statistic();
        self.ratio_statistic();
    }

    pub async fn real_trade(&mut self, data: general_data::MarketData) {
        if let Some(kline) = data.get_kline() {
            if self.symbols.contains(data.get_symbol()) {
                let kline = kline.clone();
                let symbol = data.get_symbol().to_string();
                self.real_kline_data.insert(symbol, kline);
                if self.real_kline_data.len() == self.symbols.len() {
                    let _ = self
                        .strategy
                        .on_schedule(&self.real_kline_data, &self.portfolio);
                    self.real_kline_data.clear();
                }
            }
        }
        // todo!("real trade");
    }

    pub async fn paper_trade(&mut self, data: general_data::MarketData) {
        if let Some(kline) = data.get_kline() {
            if self.symbols.contains(data.get_symbol()) {
                let kline = kline.clone();
                let symbol = data.get_symbol().to_string();
                self.real_kline_data.insert(symbol, kline);
                if self.real_kline_data.len() == self.symbols.len() {
                    if let Some(orders) = self
                        .strategy
                        .on_schedule(&self.real_kline_data, &self.portfolio)
                    {
                        match self.order_parser.convert_paper_order(
                            &orders,
                            &self.real_kline_data,
                            &self.portfolio,
                        ) {
                            Ok(orders) => {
                                let mut orders = orders;
                                match self.portfolio.make_orders(&mut orders) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!("error: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("error: {}", e);
                            }
                        }
                    }

                    self.portfolio
                        .update_market_price(self.get_format_close(&self.real_kline_data));
                    self.pnl_records.push(PnlRecord::new(
                        self.real_kline_data
                            .get(&self.symbols[0])
                            .unwrap()
                            .get_open_time(),
                        self.portfolio.get_pnl(),
                        self.portfolio.get_total_value() / self.portfolio.get_starting_cash(),
                    ));
                }
            }
        }
    }

    pub async fn init_trade(&mut self) {
        self.init_pure_historical_data().await;
        let format_klines = self.format_his_klines();
        let mut tmp_orders = HashMap::new();

        for klines in format_klines {
            if tmp_orders.len() != 0 {
                match self.order_parser.convert_backetst_order(
                    &tmp_orders,
                    &klines,
                    &self.portfolio,
                ) {
                    Ok(orders) => {
                        let mut orders = orders;
                        match self.portfolio.make_orders(&mut orders) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("error: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("error: {}", e);
                        break;
                    }
                }
            }

            if let Some(orders) = self.strategy.on_schedule(&klines, &self.portfolio) {
                tmp_orders = orders;
            } else {
                tmp_orders.clear();
            }
            self.portfolio
                .update_market_price(self.get_format_close(&klines));
            self.pnl_records.push(PnlRecord::new(
                klines.get(&self.symbols[0]).unwrap().get_open_time(),
                self.portfolio.get_pnl(),
                self.portfolio.get_total_value() / self.portfolio.get_starting_cash(),
            ));
        }
    }

    pub fn get_kline(&self, symbol: &str) -> Option<&Vec<general_data::Kline>> {
        self.kline_data.get(symbol)
    }
}
