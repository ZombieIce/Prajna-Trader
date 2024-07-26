#[derive(Debug, Clone, Copy)]
pub struct PriceLevel {
    price: f64,
    quantity: f64,
}

impl PriceLevel {
    pub fn new(price: f64, quantity: f64) -> Self {
        PriceLevel { price, quantity }
    }

    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }
}

#[derive(Debug, Clone)]
pub struct Depth {
    asks: Vec<PriceLevel>,
    bids: Vec<PriceLevel>,
}

impl Depth {
    pub fn new(asks: Vec<PriceLevel>, bids: Vec<PriceLevel>) -> Self {
        Depth { asks, bids }
    }

    pub fn get_asks(&self) -> &Vec<PriceLevel> {
        &self.asks
    }

    pub fn get_bids(&self) -> &Vec<PriceLevel> {
        &self.bids
    }

    pub fn get_best_bid(&self) -> PriceLevel {
        self.bids[0].clone()
    }

    pub fn get_best_ask(&self) -> PriceLevel {
        self.asks.last().unwrap().clone()
    }

    pub fn get_mid_price(&self) -> f64 {
        (self.get_best_bid().get_price() + self.get_best_ask().get_price()) / 2.0
    }
}