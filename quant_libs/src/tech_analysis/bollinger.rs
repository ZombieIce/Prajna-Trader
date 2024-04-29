use super::ma::SMA;
use crate::math_tools;
pub struct Bollinger {
    ma: SMA,
    period: usize,
    multiplier: f64,
    data: Vec<f64>,
}

impl Bollinger {
    pub fn new(period: usize, multiplier: f64) -> Self {
        Self {
            ma: SMA::new(period),
            period,
            multiplier,
            data: Vec::new(),
        }
    }

    pub fn add(&mut self, val: f64) {
        self.ma.add(val);
        self.data.push(val);
        if self.data.len() > self.period {
            self.data.remove(0);
        }
    }

    pub fn get(&self) -> (f64, f64, f64) {
        let mean = self.ma.get();
        let std = math_tools::std(&self.data);
        (
            mean,
            mean + std * self.multiplier,
            mean - std * self.multiplier,
        )
    }
}
