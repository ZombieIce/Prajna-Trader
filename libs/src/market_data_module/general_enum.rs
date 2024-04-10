pub enum MarketType {
    SPOT,
    FUTURES,
}

impl MarketType {
    pub fn get_market_type(&self) -> String {
        match self {
            MarketType::SPOT => "spot".to_string(),
            MarketType::FUTURES => "futures".to_string(),
        }
    }
}

pub enum Interval {
    Min5,
    Min10,
    Min15,
    Min30,
    Hour1,
    Hour2,
    Hour4,
    Day,
}

impl Interval {
    pub fn get_divider(&self) -> usize {
        match self {
            Interval::Min5 => 0,
            Interval::Min10 => 2,
            Interval::Min15 => 3,
            Interval::Min30 => 6,
            Interval::Hour1 => 12,
            Interval::Hour2 => 24,
            Interval::Hour4 => 48,
            Interval::Day => 12 * 24,
        }
    }
}

pub enum Side {
    BUY,
    SELL,
}
