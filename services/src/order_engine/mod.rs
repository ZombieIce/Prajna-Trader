pub struct OrderEngine {
    url: String,
    api_key: String,
    secret_key: String,
}

impl Default for OrderEngine {
    fn default() -> Self {
        Self {
            url: "https://fapi.binance.com".to_owned(),
            api_key: "".to_owned(),
            secret_key: "".to_owned(),
        }
    }
}


