use base_libs::market_data_module::general_data::Kline;

pub struct SuperTrend {
    period: usize,
    multiplier: f64,
    atr: f64,
    ma: f64,
    cur_trend: i8,
    cur_up: f64,
    cur_dn: f64,
    last_up: f64,
    last_dn: f64,
}

impl SuperTrend {
    pub fn new(period: usize, multiplier: f64) -> Self {
        Self {
            period,
            multiplier,
            atr: 0.0,
            ma: 0.0,
            cur_trend: 1,
            cur_up: 0.0,
            cur_dn: 0.0,
            last_up: 0.0,
            last_dn: 0.0,
        }
    }

    pub fn add(&mut self, kline: Kline) {
        self.last_dn = self.cur_dn;
        self.last_up = self.cur_up;
        let tr = (kline.get_high() - kline.get_low())
            .max(kline.get_high() - kline.get_close())
            .max(kline.get_close() - kline.get_low());

        if self.atr == 0.0 {
            self.atr = tr;
        } else {
            self.atr = (self.atr * (self.period - 1) as f64 + tr) / self.period as f64;
        }

        if self.ma == 0.0 {
            self.ma = (kline.get_high() + kline.get_low()) / 2.0;
        } else {
            self.ma = (self.ma * (self.period - 1) as f64
                + (kline.get_high() + kline.get_low()) / 2.0)
                / self.period as f64;
        }

        self.ma = (kline.get_high() + kline.get_low()) / 2.0;

        self.cur_up = self.ma - self.multiplier * self.atr;
        self.cur_dn = self.ma + self.multiplier * self.atr;

        self.cur_up = if kline.get_close() > self.last_up {
            self.cur_up.max(self.last_up)
        } else {
            self.cur_up
        };

        self.cur_dn = if kline.get_close() < self.last_dn {
            self.cur_dn.min(self.last_dn)
        } else {
            self.cur_dn
        };

        self.cur_trend = if self.cur_trend == -1 && kline.get_close() > self.last_dn {
            1
        } else if self.cur_trend == 1 && kline.get_close() < self.last_up {
            -1
        } else {
            self.cur_trend
        };
    }

    pub fn get(&self) -> (f64, f64, i8) {
        (self.cur_up, self.cur_dn, self.cur_trend)
    }
}
