use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Kline {
    open_time: i64,
    close_time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    number_of_trades: i64,
    active_buy_asset_volume: f64,
    active_buy_quote_volume: f64,
}

impl Default for Kline {
    fn default() -> Self {
        Kline {
            open_time: 0,
            close_time: 0,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
            number_of_trades: 0,
            active_buy_asset_volume: 0.0,
            active_buy_quote_volume: 0.0,
        }
    }
}

impl Kline {
    pub fn new(
        open_time: i64,
        close_time: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        number_of_trades: i64,
        active_buy_asset_volume: f64,
        active_buy_quote_volume: f64,
    ) -> Self {
        Kline {
            open_time,
            close_time,
            open,
            high,
            low,
            close,
            volume,
            number_of_trades,
            active_buy_asset_volume,
            active_buy_quote_volume,
        }
    }

    pub fn combine(&self, other: &Kline) -> Kline {
        Kline {
            open_time: self.open_time,
            close_time: other.close_time,
            open: self.open,
            high: self.high.max(other.high),
            low: self.low.min(other.low),
            close: other.close,
            volume: self.volume + other.volume,
            number_of_trades: self.number_of_trades + other.number_of_trades,
            active_buy_asset_volume: self.active_buy_asset_volume + other.active_buy_asset_volume,
            active_buy_quote_volume: self.active_buy_quote_volume + other.active_buy_quote_volume,
        }
    }

    pub fn get_open_time(&self) -> i64 {
        self.open_time
    }

    pub fn get_open(&self) -> f64 {
        self.open
    }

    pub fn get_high(&self) -> f64 {
        self.high
    }

    pub fn get_low(&self) -> f64 {
        self.low
    }

    pub fn get_close(&self) -> f64 {
        self.close
    }

    pub fn get_volume(&self) -> f64 {
        self.volume
    }

    pub fn get_number_of_trades(&self) -> i64 {
        self.number_of_trades
    }

    pub fn get_active_buy_asset_volume(&self) -> f64 {
        self.active_buy_asset_volume
    }

    pub fn get_active_buy_quote_volume(&self) -> f64 {
        self.active_buy_quote_volume
    }

    pub fn get_close_time(&self) -> i64 {
        self.close_time
    }
}