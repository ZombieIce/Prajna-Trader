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
