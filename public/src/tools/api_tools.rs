use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::base_model::api_model::{MarketData, MarketDataType};
use crate::base_model::trade_model::order_model::Order;
use crate::base_model::trade_model::position_model::Position;
use crate::exchange_model::binance_model::ws_data::{self, WsOrderEvent};
use crate::strategy_model::strategy_portfolio::Balance;

fn get_kline_topic(symbol: &str) -> String {
    format!("{}@kline_5m", symbol)
}

fn get_depth_topic(symbol: &str) -> String {
    format!("{}@depth5", symbol)
}

pub fn parse_market_data(data: &str) -> Option<MarketData> {
    if let Ok(event) = serde_json::from_str::<ws_data::WsEvent>(&data) {
        let (stream, data) = (event.stream, event.data);
        let stream_parts: Vec<&str> = stream.split('@').collect();
        if stream_parts.len() == 2 {
            let symbol = stream_parts[0];
            if stream == get_kline_topic(symbol) {
                if let Ok(ws_event_kline) = serde_json::from_value::<ws_data::WsEventKline>(data) {
                    let ws_kline = ws_event_kline.k;
                    if ws_kline.is_final() {
                        let kline = ws_kline.convert_to_standard_kline();
                        return Some(MarketData::new(
                            symbol.to_string(),
                            MarketDataType::Kline(kline),
                        ));
                    }
                }
            } else if stream == get_depth_topic(symbol) {
                if let Ok(ws_depth) = serde_json::from_value::<ws_data::WsDepth>(data) {
                    let depth = ws_depth.convert_to_standard_depth();
                    return Some(MarketData::new(
                        symbol.to_string(),
                        MarketDataType::Depth(depth),
                    ));
                }
            }
        }
    }
    None
}

pub fn parse_ws_order(data: &str) -> Option<Order> {
    if let Ok(order_data) = serde_json::from_str::<WsOrderEvent>(&data) {
        let ws_order = order_data.get_order();
        let order = ws_order.convert_to_standard_order();
        return Some(order);
    }
    None
}

pub fn parse_ws_account(data: &str) -> (Option<Balance>, Option<Vec<Position>>) {
    if let Ok(account_data) = serde_json::from_str::<ws_data::WsAccountEvent>(&data) {
        let update_time = account_data.get_update_time();
        let data = account_data.get_data();
        let balance_data = data.get_balance();
        let mut balance_res = Balance::default();
        let mut pos_res: Vec<Position> = Vec::new();
        if balance_data.len() > 0 {
            balance_res.set_balance(balance_data[0].get_balance());
            balance_res.set_update_time(update_time);
        }
        let pos_data = data.get_position();

        if pos_data.len() > 0 {
            for pos in pos_data {
                let cur_standard_pos = pos.convert_to_standard_position(update_time);
                pos_res.push(cur_standard_pos);
            }
        }
        return (Some(balance_res), Some(pos_res));
    }
    (None, None)
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
    fn test_create_signature() {
        let secret_key = "2b5eb11e18796d12d88f13dc27dbbd02c2cc51ff7059765ed9821957d82bb4d9";
        let params = "symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=9000&timeInForce=GTC&recvWindow=5000&timestamp=1591702613943";
        let res = get_signature(secret_key, params);
        println!("{}", res);
    }

    #[test]
    fn test_parse_ws_order() {
        let mut data = "{\"e\":\"ORDER_TRADE_UPDATE\",\"T\":1721975806250,\"E\":1721975806250,\"o\":{\"s\":\"BTCUSDT\",\"c\":\"test_BTCUSDT_1721975805809\",\"S\":\"BUY\",\"o\":\"LIMIT\",\"f\":\"GTC\",\"q\":\"0.020\",\"p\":\"67000\",\"ap\":\"0\",\"sp\":\"0\",\"x\":\"NEW\",\"X\":\"NEW\",\"i\":379584107093,\"l\":\"0\",\"z\":\"0\",\"L\":\"0\",\"n\":\"0\",\"N\":\"USDT\",\"T\":1721975806250,\"t\":0,\"b\":\"1340\",\"a\":\"0\",\"m\":false,\"R\":false,\"wt\":\"CONTRACT_PRICE\",\"ot\":\"LIMIT\",\"ps\":\"BOTH\",\"cp\":false,\"rp\":\"0\",\"pP\":false,\"si\":0,\"ss\":0,\"V\":\"NONE\",\"pm\":\"NONE\",\"gtd\":0}}";
        if let Some(order) = parse_ws_order(data) {
            println!("{:?}", order);
        }
        data = "{\"e\":\"ORDER_TRADE_UPDATE\",\"T\":1721975806250,\"E\":1721975806250,\"o\":{\"s\":\"BTCUSDT\",\"c\":\"test_BTCUSDT_1721975805809\",\"S\":\"BUY\",\"o\":\"LIMIT\",\"f\":\"GTC\",\"q\":\"0.020\",\"p\":\"67000\",\"ap\":\"66998.30000\",\"sp\":\"0\",\"x\":\"TRADE\",\"X\":\"PARTIALLY_FILLED\",\"i\":379584107093,\"l\":\"0.017\",\"z\":\"0.017\",\"L\":\"66998.30\",\"n\":\"0.56948555\",\"N\":\"USDT\",\"T\":1721975806250,\"t\":5203046333,\"b\":\"201\",\"a\":\"0\",\"m\":false,\"R\":false,\"wt\":\"CONTRACT_PRICE\",\"ot\":\"LIMIT\",\"ps\":\"BOTH\",\"cp\":false,\"rp\":\"0\",\"pP\":false,\"si\":0,\"ss\":0,\"V\":\"NONE\",\"pm\":\"NONE\",\"gtd\":0}}";
        if let Some(order) = parse_ws_order(data) {
            println!("{:?}", order);
        }
        data = "{\"e\":\"ORDER_TRADE_UPDATE\",\"T\":1721975806250,\"E\":1721975806250,\"o\":{\"s\":\"BTCUSDT\",\"c\":\"test_BTCUSDT_1721975805809\",\"S\":\"BUY\",\"o\":\"LIMIT\",\"f\":\"GTC\",\"q\":\"0.020\",\"p\":\"67000\",\"ap\":\"66998.30000\",\"sp\":\"0\",\"x\":\"TRADE\",\"X\":\"FILLED\",\"i\":379584107093,\"l\":\"0.003\",\"z\":\"0.020\",\"L\":\"66998.30\",\"n\":\"0.10049745\",\"N\":\"USDT\",\"T\":1721975806250,\"t\":5203046334,\"b\":\"0\",\"a\":\"0\",\"m\":false,\"R\":false,\"wt\":\"CONTRACT_PRICE\",\"ot\":\"LIMIT\",\"ps\":\"BOTH\",\"cp\":false,\"rp\":\"0\",\"pP\":false,\"si\":0,\"ss\":0,\"V\":\"NONE\",\"pm\":\"NONE\",\"gtd\":0}}";
        if let Some(order) = parse_ws_order(data) {
            println!("{:?}", order);
        }
    }

    #[test]
    fn test_parse_ws_account() {
        let data = "{\"e\":\"ACCOUNT_UPDATE\",\"T\":1721976305145,\"E\":1721976305145,\"a\":{\"B\":[{\"a\":\"USDT\",\"wb\":\"2102.58528451\",\"cw\":\"2102.58528451\",\"bc\":\"0\"}],\"P\":[{\"s\":\"BTCUSDT\",\"pa\":\"0\",\"ep\":\"0\",\"cr\":\"140.70390000\",\"up\":\"0\",\"mt\":\"cross\",\"iw\":\"0\",\"ps\":\"BOTH\",\"ma\":\"USDT\",\"bep\":\"0\"}],\"m\":\"ORDER\"}}";
        parse_ws_account(data);
    }
}
