use super::ma::RMA;

pub struct RSI {
    previous_data: f64,
    up_rma: RMA,
    dn_rma: RMA,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self {
            previous_data: 0.0,
            up_rma: RMA::new(period),
            dn_rma: RMA::new(period),
        }
    }

    pub fn add(&mut self, data: f64) {
        if self.previous_data == 0.0 {
            self.previous_data = data;
            return;
        }
        let diff = data - self.previous_data;
        let up = if diff > 0.0 { diff } else { 0.0 };
        let down = if diff < 0.0 { -diff } else { 0.0 };

        self.up_rma.add(up);
        self.dn_rma.add(down);
        self.previous_data = data;
    }

    pub fn get(&self) -> f64 {
        if self.up_rma.get() == 0.0 {
            return 0.0;
        }

        if self.dn_rma.get() == 0.0 {
            return 100.0;
        }

        100.0 - 100.0 / (1.0 + self.up_rma.get() / self.dn_rma.get())
    }
}
