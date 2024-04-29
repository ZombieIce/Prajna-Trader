use base_libs::base_strategy::base_strategy::BaseStrategy;
use base_libs::base_strategy::common_module::TargetPosition;
use quant_libs::tech_analysis::bollinger;
use tracing::info;

pub struct BollingerBandStrategy {
    strategy_name: String,
    symbol: String,
    bollinger: bollinger::Bollinger,
    cur_trend: i8,
    last_close: f64,
    last_mean: f64,
}

impl BollingerBandStrategy {
    pub fn new(strategy_name: String, symbol: String, period: usize, multiplier: f64) -> Self {
        Self {
            strategy_name,
            symbol,
            bollinger: bollinger::Bollinger::new(period, multiplier),
            cur_trend: 0,
            last_close: 0.0,
            last_mean: 0.0,
        }
    }
}

impl BaseStrategy for BollingerBandStrategy {
    fn on_schedule(
        &mut self,
        klines: &std::collections::HashMap<
            String,
            base_libs::market_data_module::general_data::Kline,
        >,
        portfolio: &base_libs::base_strategy::portfolio::Portfolio,
    ) -> Option<std::collections::HashMap<String, TargetPosition>> {
        let mut res = std::collections::HashMap::new();
        let kline = klines.get(&self.symbol).unwrap();
        self.bollinger.add(kline.get_close());

        let (mean, upper, lower) = self.bollinger.get();
        info!(
            "{} update mean: {}, upper: {}, lower: {}",
            self.get_strategy_name(),
            mean,
            upper,
            lower
        );

        if kline.get_close() > upper {
            if self.cur_trend != 1 {
                self.cur_trend = 1;
                info!(
                    "{}: upper band {} breakout with close: {}",
                    self.get_strategy_name(),
                    upper,
                    kline.get_close()
                );
            }
        } else if kline.get_close() < lower {
            if self.cur_trend != -1 {
                self.cur_trend = -1;
                info!(
                    "{}: lower band {} breakout with close: {}",
                    self.get_strategy_name(),
                    lower,
                    kline.get_close()
                );
            }
        }

        if let Some(current_position) = portfolio.get_position(&self.symbol) {
            let mut new_pos = current_position.get_qty();
            if current_position.get_qty() > 0.0 && self.cur_trend == -1 {
                info!(
                    "{}: close below lower band, CLOSE LONG POSITION",
                    self.get_strategy_name(),
                );

                new_pos = 0.0;
            } else if current_position.get_qty() < 0.0 && self.cur_trend == 1 {
                info!(
                    "{}: close above upper band, CLOSE SHORT POSITION",
                    self.get_strategy_name(),
                );
                new_pos = 0.0;
            }

            if current_position.get_qty() == 0.0 {
                if self.cur_trend == 1
                    && self.last_close < self.last_mean
                    && kline.get_close() > mean
                    && self.last_close != 0.0
                    && mean != 0.0
                {
                    let available_cash = portfolio.get_available_cash();
                    new_pos = available_cash * 0.3 / kline.get_close();
                    info!(
                        "{}: close above mean, OPEN LONG POSITION",
                        self.get_strategy_name()
                    );
                } else if self.cur_trend == -1
                    && self.last_close > self.last_mean
                    && kline.get_close() < mean
                    && self.last_mean != 0.0
                {
                    let available_cash = portfolio.get_available_cash();
                    new_pos = -available_cash * 0.3 / kline.get_close();
                    info!(
                        "{}: close below mean, OPEN SHORT POSITION",
                        self.get_strategy_name()
                    );
                }
            }
            if new_pos != current_position.get_qty() {
                res.insert(self.symbol.clone(), TargetPosition::new(new_pos));
            }
        }
        self.last_close = kline.get_close();
        self.last_mean = mean;
        if res.len() > 0 {
            return Some(res);
        }
        None
    }
    fn get_strategy_name(&self) -> String {
        format!("{}_{}", self.strategy_name, self.symbol)
    }
}
