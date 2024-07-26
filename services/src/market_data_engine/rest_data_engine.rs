use crate::api_enum::FuturesApi;
use public::exchange_model::binance_model::rest_data;
use public::base_model::info_model::{SymbolInfo, ExchangeInfo};
use public::base_model::market_model::kline_model::Kline;
use public::base_enum::market_enums::MarketType;
use crate::mongo_engine::MongoEngine;



use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::Value;
use tokio;
use tracing::{error, info};

#[derive(Clone)]
pub struct RestDataEngine {
    url: String,
    request_weight_limit: i64,
    symbols: Vec<String>,
    mongo_engine: MongoEngine,
}

impl Default for RestDataEngine {
    fn default() -> Self {
        Self {
            url: "https://fapi.binance.com".to_string(),
            request_weight_limit: 0,
            symbols: vec![],
            mongo_engine: MongoEngine::default(),
        }
    }
}

impl RestDataEngine {
    fn get_api(&self, item: FuturesApi) -> String {
        let api_url = match item {
            FuturesApi::ExchangeInfo => "/fapi/v1/exchangeInfo".to_string(),
            FuturesApi::Klines => "/fapi/v1/klines".to_string(),
        };
        format!("{}{}", self.url, api_url)
    }

    fn parse_rate_limit(&self, rate_limits: &Vec<Value>) -> i64 {
        for rate_limit in rate_limits {
            if let Some(specified_rate_limit) = rate_limit.as_object() {
                if let Some(rate_limit_type) = specified_rate_limit.get("rateLimitType") {
                    if rate_limit_type.as_str().unwrap() == "REQUEST_WEIGHT" {
                        if let Some(limit) = specified_rate_limit.get("limit").unwrap().as_i64() {
                            return limit;
                        }
                    }
                }
            }
        }
        0
    }

    fn parse_symbol_info(&self, symbol_info: &Value) -> SymbolInfo {
        let symbol_name = symbol_info.get("symbol").unwrap().as_str().unwrap();
        let mut price_precision = symbol_info.get("pricePrecision").unwrap().as_i64().unwrap();
        let mut notional = 0.0;
        let mut min_qty = 0.0;
        let mut max_qty = 0.0;
        let quantity_precision = symbol_info
            .get("quantityPrecision")
            .unwrap()
            .as_i64()
            .unwrap();
        let filters = symbol_info.get("filters").unwrap().as_array().unwrap();
        for filter in filters {
            if let Some(filter_type) = filter.as_object().unwrap().get("filterType") {
                if filter_type.as_str().unwrap() == "PRICE_FILTER" {
                    let tick_size = filter
                        .as_object()
                        .unwrap()
                        .get("tickSize")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    let tick_size = tick_size.parse::<f64>().unwrap();
                    price_precision = -tick_size.log10() as i64;
                }
                if filter_type.as_str().unwrap() == "MIN_NOTIONAL" {
                    let cur_notional = filter
                        .as_object()
                        .unwrap()
                        .get("notional")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    notional = cur_notional.parse::<f64>().unwrap();
                }
                if filter_type.as_str().unwrap() == "MARKET_LOT_SIZE" {
                    let cur_min_qty = filter
                        .as_object()
                        .unwrap()
                        .get("minQty")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    min_qty = cur_min_qty.parse::<f64>().unwrap();
                }
                if filter_type.as_str().unwrap() == "MARKET_LOT_SIZE" {
                    let cur_max_qty = filter
                        .as_object()
                        .unwrap()
                        .get("maxQty")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    max_qty = cur_max_qty.parse::<f64>().unwrap();
                }
            }
        }
        SymbolInfo::new(
            symbol_name.to_string(),
            price_precision,
            quantity_precision,
            notional,
            min_qty,
            max_qty,
        )
    }

    pub fn subscribe_symbols(&mut self, symbols: &Vec<String>) {
        self.symbols = symbols.to_vec();
    }

    pub async fn fetch_exchange_info(&self) -> Option<ExchangeInfo> {
        match reqwest::get(self.get_api(FuturesApi::ExchangeInfo)).await {
            Ok(res) => match res.text().await {
                Ok(text) => {
                    if let Ok(exchange_info) =
                        serde_json::from_str::<rest_data::ExchangeInfo>(&text)
                    {
                        let mut limit: i64 = 0;
                        let mut symbol_infos: Vec<SymbolInfo> = vec![];
                        let server_time = exchange_info.server_time;
                        if let Some(rate_limits) = exchange_info.rate_limits.as_array() {
                            limit = self.parse_rate_limit(rate_limits);
                        }
                        if let Some(symbols) = exchange_info.symbols.as_array() {
                            symbol_infos = symbols
                                .iter()
                                .map(|x| self.parse_symbol_info(x))
                                .filter(|y| self.symbols.contains(&y.get_symbol().to_lowercase()))
                                .collect::<Vec<SymbolInfo>>();
                        }
                        Some(ExchangeInfo::new(
                            "binance".to_string(),
                            symbol_infos,
                            MarketType::FUTURES,
                            limit,
                            server_time,
                        ))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    error!("{}", e);
                    None
                }
            },
            Err(e) => {
                error!("{}", e);
                None
            }
        }
    }

    async fn pure_update_exchange_info(&mut self, msg: &str) {
        if let Some(exchange_info) = self.fetch_exchange_info().await {
            println!("{:?}", exchange_info);
            self.request_weight_limit = exchange_info.get_rest_limit_rate();

            match self.mongo_engine.update_exchange_info(&exchange_info).await {
                Ok(_) => {
                    info!("{} exchange info successfully", msg);
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }

    pub async fn update_exchange_info(&mut self) {
        let delta_time = 1000 * 60 * 60 * 24;
        match self.mongo_engine.get_exchange_info().await {
            Some(prev_exchange_info) => {
                let now_timestamp = chrono::Utc::now().timestamp_millis();
                if now_timestamp - prev_exchange_info.get_server_time() >= delta_time {
                    self.pure_update_exchange_info("update").await;
                } else {
                    self.request_weight_limit = prev_exchange_info.get_rest_limit_rate();
                }
            }
            None => self.pure_update_exchange_info("init").await,
        }
    }

    pub async fn fetch_single_batch_his_kline(
        &self,
        symbol: &str,
        start_time: i64,
    ) -> Option<Vec<Kline>> {
        let api_url = self.get_api(FuturesApi::Klines);
        let request_url = format!(
            "{}?symbol={}&interval=5m&startTime={}&limit=1000",
            api_url, symbol, start_time
        );
        match reqwest::get(request_url.clone()).await {
            Ok(res) => {
                let headers = res.headers();
                for (name, value) in headers {
                    if name.to_string() == "x-mbx-used-weight-1m".to_string() {
                        let cur_limit = value.to_str().unwrap().parse::<i64>().unwrap();
                        if cur_limit >= self.request_weight_limit * 8 / 10 {
                            // sleep 10 seconds
                            info!("Request weight limit reached, sleep 10 seconds");
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        }
                    }
                }
                match res.text().await {
                    Ok(data) => {
                        if let Ok(klines) = serde_json::from_str::<Vec<Vec<Value>>>(&data) {
                            let format_kline: Vec<Kline> = klines
                                .iter()
                                .map(|x| {
                                    Kline::new(
                                        x[0].as_i64().unwrap(),
                                        x[6].as_i64().unwrap(),
                                        x[1].as_str().unwrap().parse::<f64>().unwrap(),
                                        x[2].as_str().unwrap().parse::<f64>().unwrap(),
                                        x[3].as_str().unwrap().parse::<f64>().unwrap(),
                                        x[4].as_str().unwrap().parse::<f64>().unwrap(),
                                        x[5].as_str().unwrap().parse::<f64>().unwrap(),
                                        x[8].as_i64().unwrap(),
                                        x[9].as_str().unwrap().parse::<f64>().unwrap(),
                                        x[10].as_str().unwrap().parse::<f64>().unwrap(),
                                    )
                                })
                                .collect();
                            Some(format_kline)
                        } else {
                            error!("Failed to parse kline data");
                            None
                        }
                    }
                    Err(e) => {
                        error!("{}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect url {} {}", request_url, e);
                None
            }
        }
    }

    pub async fn fetch_his_kline(&self, symbol: &str) {
        let date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let naive_datetime = NaiveDateTime::new(date, time);
        let mut start_time = naive_datetime.and_utc().timestamp_millis();
        match self.mongo_engine.fetch_latest_kline(symbol).await {
            Ok(last_stored_kline) => {
                if let Some(last_stored_kline) = last_stored_kline {
                    start_time = last_stored_kline.get_close_time() + 1;
                }

                let mut is_finished = false;
                loop {
                    match self.fetch_single_batch_his_kline(symbol, start_time).await {
                        Some(batch_klines) => {
                            let mut cur_klines = batch_klines.clone();
                            start_time = cur_klines.last().unwrap().get_close_time() + 1;

                            if start_time >= chrono::Utc::now().timestamp_millis() {
                                info!("{} kline data done !", symbol);
                                is_finished = true;
                                cur_klines = cur_klines[..cur_klines.len() - 1].to_vec();
                            }

                            if cur_klines.len() != 0 {
                                match self.mongo_engine.insert_kline(symbol, &cur_klines).await {
                                    Ok(_) => {
                                        let start_datetime =
                                            DateTime::from_timestamp(start_time / 1000, 0).unwrap();
                                        info!(
                                            "Inserted {} kline data with start_time: {:#?} count: {}",
                                            symbol, start_datetime, cur_klines.len()
                                        );
                                    }
                                    Err(e) => {
                                        error!("Failed to insert_kline {} {}", symbol, e);
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                        None => {
                            error!("Failed to fetch {} kline data", symbol);
                        }
                    }
                    if is_finished {
                        break;
                    }
                }
            }
            Err(e) => {
                error!("failed to fetch_latest_kline {} Error: {}", symbol, e);
            }
        }
    }

    pub async fn start(&self) {
        let symbol_list = self.symbols.clone();
        let mut handles = Vec::new();
        for symbol in symbol_list {
            let cur_engine = self.clone();
            let handle = tokio::spawn(async move {
                cur_engine.fetch_his_kline(&symbol).await;
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
