#[derive(Debug, Clone, Copy)]
pub struct TargetPosition(f64);
impl TargetPosition {
    pub fn new(qty: f64) -> Self {
        Self(qty)
    }

    pub fn get_position(&self) -> f64 {
        self.0
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
