pub fn round_to_precision(value: f64, precision: i64) -> f64 {
    let multiplier = 10f64.powi(precision as i32);
    (value * multiplier).round() / multiplier
}
