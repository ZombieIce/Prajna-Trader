use hmac::{Hmac, Mac};
use sha2::Sha256;

use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct Settings {
    api_key: String,
    secret_key: String,
}

impl Settings {
    pub fn get_api_key(&self) -> String {
        self.api_key.clone()
    }

    pub fn get_secret_key(&self) -> String {
        self.secret_key.clone()
    }
}

pub fn load_settings(path: &str) -> Settings {
    let mut file = File::open(path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let settings: Settings = serde_yaml::from_str(&contents).expect("Unable to parse YAML");

    settings
}

pub fn round_to_precision(value: f64, precision: i64) -> f64 {
    let multiplier = 10f64.powi(precision as i32);
    (value * multiplier).round() / multiplier
}

pub fn get_signature(secret_key: &str, params: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(params.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_to_precision() {
        let value = 123.456789;
        let precision = 2;
        let expected_result = 123.46;

        let result = round_to_precision(value, precision);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_create_signature() {
        let secret_key = "2b5eb11e18796d12d88f13dc27dbbd02c2cc51ff7059765ed9821957d82bb4d9";
        let params = "symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=9000&timeInForce=GTC&recvWindow=5000&timestamp=1591702613943";
        let res = get_signature(secret_key, params);
        println!("{}", res);
    }
}
