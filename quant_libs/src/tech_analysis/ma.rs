pub struct EMA {
    alpha: f64,
    last_ema: f64,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        Self {
            alpha: 2.0 / (period as f64 + 1.0),
            last_ema: 0.0,
        }
    }

    pub fn add(&mut self, val: f64) {
        if self.last_ema == 0.0 {
            self.last_ema = val;
        } else {
            self.last_ema = self.alpha * val + (1.0 - self.alpha) * self.last_ema;
        }
    }

    pub fn get(&self) -> f64 {
        self.last_ema
    }
}

pub struct SMA {
    period: usize,
    data: Vec<f64>,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            data: Vec::new(),
        }
    }

    pub fn add(&mut self, val: f64) {
        self.data.push(val);
        if self.data.len() > self.period {
            self.data.remove(0);
        }
    }

    pub fn get(&self) -> f64 {
        if self.data.len() != self.period {
            return 0.0;
        }
        self.data.iter().sum::<f64>() / self.data.len() as f64
    }
}
