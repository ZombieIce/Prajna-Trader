pub mod base_strategy;
pub mod portfolio;
pub mod strategy_error;

// struct TestStrategy {
//     strategy_name: String,
//     prev_close: HashMap<String, f64>,
//     position_ratio: f64,
// }

// impl TestStrategy {
//     pub fn new(strategy_name: String) -> Self {
//         Self {
//             strategy_name,
//             prev_close: HashMap::new(),
//             position_ratio: 0.1,
//         }
//     }
// }

// impl base_strategy::BaseStrategy for TestStrategy {
//     fn get_strategy_name(&self) -> String {
//         self.strategy_name.clone()
//     }

//     fn on_schedule(
//         &mut self,
//         klines: &HashMap<String, general_data::Kline>,
//         portfolio: &portfolio::Portfolio,
//     ) -> HashMap<String, TargetPosition> {
//         println!("{:#?}", portfolio);

//         let mut rt_map: HashMap<String, f64> = HashMap::new();

//         for (s, k) in klines.iter() {
//             if let Some(prev_close) = self.prev_close.get(s) {
//                 let ret = (k.get_close() - prev_close) / prev_close;
//                 rt_map.insert(s.clone(), ret);
//             }
//             self.prev_close.insert(s.clone(), k.get_close());
//         }

//         // sort by return
//         let mut tartget_position_map: HashMap<String, TargetPosition> = HashMap::new();
//         if rt_map.len() != 0 {
//             let mut sorted_rt_map: Vec<(&String, &f64)> = rt_map.iter().collect();
//             sorted_rt_map.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

//             let cur_cash = portfolio.get_total_value();
//             tartget_position_map.insert(
//                 sorted_rt_map[0].0.clone(),
//                 TargetPosition::new(
//                     cur_cash * self.position_ratio
//                         / klines.get(sorted_rt_map[0].0).unwrap().get_close(),
//                 ),
//             );
//             tartget_position_map.insert(
//                 sorted_rt_map[1].0.clone(),
//                 TargetPosition::new(
//                     cur_cash * self.position_ratio
//                         / klines.get(sorted_rt_map[1].0).unwrap().get_close(),
//                 ),
//             );
//             tartget_position_map.insert(
//                 sorted_rt_map[sorted_rt_map.len() - 1].0.clone(),
//                 TargetPosition::new(
//                     -cur_cash * self.position_ratio
//                         / klines
//                             .get(sorted_rt_map[sorted_rt_map.len() - 1].0)
//                             .unwrap()
//                             .get_close(),
//                 ),
//             );
//             tartget_position_map.insert(
//                 sorted_rt_map[sorted_rt_map.len() - 2].0.clone(),
//                 TargetPosition::new(
//                     -cur_cash * self.position_ratio
//                         / klines
//                             .get(sorted_rt_map[sorted_rt_map.len() - 2].0)
//                             .unwrap()
//                             .get_close(),
//                 ),
//             );
//             println!("{:#?}", tartget_position_map);
//         }

//         tartget_position_map
//     }
// }

// // test
// #[cfg(test)]
// mod test {
//     use chrono::DateTime;

//     use super::base_strategy::StrategyContext;
//     use super::TestStrategy;
//     use crate::market_data_module::general_enum;

//     #[tokio::test]
//     async fn test_base_strategy() {
//         let start_date_timestamp = DateTime::parse_from_rfc3339("2024-03-01T00:00:00Z")
//             .unwrap()
//             .timestamp_millis();
//         let test_strategy = TestStrategy::new("TestStrategy".to_string());
//         let mut strategy = StrategyContext::new(
//             vec![
//                 "btcusdt".to_string(),
//                 // "ethusdt".to_string(),
//                 // "solusdt".to_string(),
//                 // "dogeusdt".to_string(),
//                 // "bnbusdt".to_string(),
//                 // "maticusdt".to_string(),
//                 // "adausdt".to_string(),
//                 // "avaxusdt".to_string(),
//                 // "uniusdt".to_string(),
//                 // "dotusdt".to_string(),
//             ],
//             10000.0,
//             start_date_timestamp,
//             general_enum::Interval::Hour1,
//             20.0,
//             true,
//             Box::new(test_strategy),
//         );
//         strategy.start().await;
//     }
// }
