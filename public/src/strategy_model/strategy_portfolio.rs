use crate::base_model::market_model::kline_model::Kline;
use crate::base_model::trade_model::order_model::Order;
use crate::base_model::trade_model::position_model::Position;
use crate::base_model::{error_model::StrategyError, info_model::SymbolInfo};
use crate::tools::time_tools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    balance: f64,
    update_time: i64,
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            balance: 0.0,
            update_time: 0,
        }
    }
}

impl Balance {
    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn get_update_time(&self) -> i64 {
        self.update_time
    }

    pub fn set_balance(&mut self, balance: f64) {
        self.balance = balance;
    }

    pub fn set_update_time(&mut self, update_time: i64) {
        self.update_time = update_time;
    }
}

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

pub struct StrategyPortfolio {
    starting_cash: f64,
    available_cash: f64,
    freezed_cash: f64,
    total_value: f64,
    leverage_rate: f64,
    fee_rate: f64,
    fee: f64,
    unrealized_pnl: f64,
    realized_pnl: f64,
    positions: HashMap<String, Position>,
    orders: HashMap<String, Vec<Order>>,
    symbol_infos: HashMap<String, SymbolInfo>,
    total_value_records: Vec<f64>,
    pnl_records: Vec<PnlRecord>,
}

impl StrategyPortfolio {
    pub fn new(starting_cash: f64, leverage_rate: f64, symbols: Vec<String>) -> Self {
        Self {
            starting_cash: starting_cash,
            available_cash: starting_cash,
            freezed_cash: 0.0,
            leverage_rate: leverage_rate,
            total_value: starting_cash,
            fee_rate: 0.0005,
            fee: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            positions: symbols
                .into_iter()
                .map(|s| (s, Position::default()))
                .collect::<HashMap<String, Position>>(),
            orders: HashMap::new(),
            symbol_infos: HashMap::new(),
            total_value_records: Vec::new(),
            pnl_records: Vec::new(),
        }
    }

    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    pub fn get_available_cash(&self) -> f64 {
        self.available_cash
    }

    pub fn get_leverage_rate(&self) -> f64 {
        self.leverage_rate
    }
    pub fn set_symbol_infos(&mut self, symbol_infos: HashMap<String, SymbolInfo>) {
        self.symbol_infos = symbol_infos;
        for (s, info) in &self.symbol_infos {
            info!(
                "Symbol: {} | min qty: {} | min notional: {}",
                s,
                info.get_min_quantity(),
                info.get_min_notional()
            );
        }
    }

    pub fn get_symbol_infos(&self) -> &HashMap<String, SymbolInfo> {
        &self.symbol_infos
    }

    fn check_insufficient_cash(&self, symbol: &str, order: &Order) -> Result<(), StrategyError> {
        if let Some(cur_pos) = self.positions.get(symbol) {
            // open new order
            if (cur_pos.get_quantity() == 0.0 || order.get_side() == cur_pos.get_side())
                && order.get_price() * order.get_qty() > self.available_cash * self.leverage_rate
            {
                let msg = format!(
                    "Insufficient cash {} to place {} order with px: {} | qty: {}",
                    self.available_cash,
                    order.get_symbol(),
                    order.get_price(),
                    order.get_qty(),
                );
                return Err(StrategyError::InsufficientCashError(msg));
            }

            if cur_pos.get_quantity() != 0.0 && cur_pos.get_side() != order.get_side() {
                let new_margin = order.get_price() * order.get_qty() / self.leverage_rate;
                if new_margin - cur_pos.get_margin() > self.available_cash {
                    let msg = format!(
                        "Insufficient cash {} to place {} order with px: {} | qty: {}",
                        self.available_cash,
                        order.get_symbol(),
                        order.get_price(),
                        order.get_qty(),
                    );
                    return Err(StrategyError::InsufficientCashError(msg));
                }
            }
        }
        Ok(())
    }

    fn check_back_test_order(&mut self, symbol: &str, order: &Order) -> Result<(), StrategyError> {
        match self.check_insufficient_cash(symbol, order) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        if order.get_price() * order.get_qty()
            <= self.symbol_infos.get(symbol).unwrap().get_min_notional()
        {
            let msg = format!(
                "Not meet min notional {} for {} order with px: {} | qty: {}",
                self.symbol_infos.get(symbol).unwrap().get_min_notional(),
                order.get_symbol(),
                order.get_price(),
                order.get_qty(),
            );
            return Err(StrategyError::OrderNotionalError(msg));
        }

        if order.get_qty() <= self.symbol_infos.get(symbol).unwrap().get_min_quantity() {
            let msg = format!(
                "Not meet min qty {} for {} order with px: {} | qty: {}",
                self.symbol_infos.get(symbol).unwrap().get_min_quantity(),
                order.get_symbol(),
                order.get_price(),
                order.get_qty(),
            );
            return Err(StrategyError::OrderQuantityError(msg));
        }
        info!(
            "Place order: {} | px: {} | qty: {} | side: {}",
            order.get_symbol(),
            order.get_price(),
            order.get_qty(),
            order.get_side().string()
        );

        if let Some(cur_pos) = self.positions.get_mut(symbol) {
            let prev_margin = cur_pos.get_margin();
            let cur_realized_pnl = cur_pos.update_order(order);
            let amt = order.get_price() * order.get_qty();
            let fee = amt * self.fee_rate;
            let delta_margin = cur_pos.get_margin() - prev_margin;

            self.fee += fee;
            self.available_cash -= delta_margin + fee;
            self.freezed_cash += delta_margin;
            self.realized_pnl += cur_realized_pnl;
            self.available_cash += cur_realized_pnl;
        }

        if let Some(cur_orders) = self.orders.get_mut(symbol) {
            cur_orders.push(order.clone());
        } else {
            self.orders.insert(symbol.to_string(), vec![order.clone()]);
        }

        Ok(())
    }

    pub fn make_back_test_order(
        &mut self,
        orders: HashMap<String, Order>,
    ) -> Result<(), StrategyError> {
        for (symbol, order) in orders {
            match self.check_back_test_order(&symbol, &order) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub fn update_back_test_market_price(&mut self, klines: &HashMap<String, Kline>) {
        for (symbol, kline) in klines {
            if let Some(cur_pos) = self.positions.get_mut(symbol) {
                cur_pos.update_market_price(kline.get_close());
            }
        }
    }

    pub fn update_back_test_value(&mut self) -> Result<(), StrategyError> {
        let mut unrealized_pnl = 0.0;
        for (_, pos) in &self.positions {
            unrealized_pnl += pos.get_unrealized_pnl();
        }
        self.unrealized_pnl = unrealized_pnl;

        let tmp_total_value =
            self.starting_cash + self.unrealized_pnl + self.realized_pnl - self.fee;
        let tmp_total_value_check = self.available_cash + self.freezed_cash + self.unrealized_pnl;
        if tmp_total_value - tmp_total_value_check < 0.0001 {
            self.total_value = tmp_total_value;
            self.total_value_records.push(self.total_value);
        } else {
            warn!(
                "Total value check failed: {} != {}",
                tmp_total_value, tmp_total_value_check
            );
            return Err(StrategyError::BacktestError(
                "Total value check failed".to_string(),
            ));
        }
        Ok(())
    }

    pub fn update_pnl_records(&mut self, timestamp: i64) {
        let pnl = self.total_value - self.starting_cash;
        let cur_pnl = PnlRecord::new(timestamp, pnl, self.total_value);
        if self.pnl_records.len() == 0 {
            info!(
                "Pnl record: timestamp: {} | pnl: {} | net value: {}",
                time_tools::get_datetime_from_timestamp(timestamp),
                cur_pnl.get_pnl(),
                cur_pnl.get_net_value()
            );
        } else {
            let last_record = self.pnl_records.last().unwrap();
            if last_record.pnl != cur_pnl.pnl || last_record.net_value != cur_pnl.net_value {
                info!(
                    "Pnl record: timestamp: {} | pnl: {} | net value: {}",
                    time_tools::get_datetime_from_timestamp(timestamp),
                    cur_pnl.get_pnl(),
                    cur_pnl.get_net_value()
                );
            }
        }
        self.pnl_records.push(cur_pnl);
    }

    pub fn show_summary(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.ratio_statistic();
        Ok(())
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

        info!("Sharpe Ratio: {}", sharpe_ratio * sqrt_ratio);
        info!("Sortino Ratio: {}", sortino_ratio * sqrt_ratio);
    }
}
