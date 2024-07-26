use futures_util::{SinkExt, StreamExt};
use public::{
    base_enum::order_enums::OrderStatus,
    tools::{api_tools, settings_tools},
};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

use crate::mongo_engine::MongoEngine;

#[derive(Debug, Clone, serde::Deserialize)]
struct ListenKey {
    #[serde(rename = "listenKey")]
    listen_key: String,
}

pub struct OrderListener {
    client: reqwest::Client,
    api_key: String,
    db_client: MongoEngine,
}

impl Default for OrderListener {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: "".to_string(),
            db_client: MongoEngine::default(),
        }
    }
}

impl OrderListener {
    fn load_settings(&mut self, path: &str) {
        let settings = settings_tools::load_settings(path);
        self.api_key = settings.get_api_key();
        info!("API_KEY: {}", self.api_key);
    }

    async fn get_usr_url(&mut self) -> Option<String> {
        let url = format!("https://fapi.binance.com/fapi/v1/listenKey");
        match self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", self.api_key.clone())
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    let listen_key = resp.text().await.unwrap();
                    if let Ok(listen_key) = serde_json::from_str::<ListenKey>(&listen_key) {
                        info!("Get ListenKey: {}", listen_key.listen_key);
                        let url = format!("wss://fstream.binance.com/ws/{}", listen_key.listen_key);
                        return Some(url);
                    }
                }
            }
            Err(e) => {
                error!("GetListenKey Error: {}", e);
            }
        }
        return None;
    }

    pub async fn start_listen(&mut self, path: &str) {
        self.load_settings(path);
        if let Some(url) = self.get_usr_url().await {
            info!("Start listen order...");
            match tokio_tungstenite::connect_async(url).await {
                Ok((ws_stream, _)) => {
                    let (mut write, mut read) = ws_stream.split();
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(msg) => match msg {
                                Message::Ping(ping) => {
                                    match write.send(Message::Pong(ping)).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!("Ping Error: {}", e);
                                        }
                                    }
                                }
                                Message::Text(data) => {
                                    if let Some(_) =
                                        data.strip_prefix("{\"e\":\"ORDER_TRADE_UPDATE\"")
                                    {
                                        if let Some(order) = api_tools::parse_ws_order(&data) {
                                            info!("Receive order message: {:?}", order);
                                            if order.get_status() == OrderStatus::Filled
                                                || order.get_status() == OrderStatus::Canceled
                                            {
                                                match self.db_client.insert_order(&order).await {
                                                    Ok(_) => {
                                                        info!("Insert order success: {:?}", order);
                                                    }
                                                    Err(e) => {
                                                        error!("Insert order error: {:?}", e);
                                                    }
                                                }
                                            }
                                        } else {
                                            error!("Parse order error: {}", data);
                                        }
                                    }

                                    if let Some(_) = data.strip_prefix("{\"e\":\"ACCOUNT_UPDATE\"")
                                    {
                                        if let (Some(balance), Some(positions)) =
                                            api_tools::parse_ws_account(&data)
                                        {
                                            info!("Receive account message: {:?}", balance);
                                            info!("Receive account message: {:?}", positions);
                                            if balance.get_balance() > 0.0 {
                                                match self.db_client.update_balance(&balance).await
                                                {
                                                    Ok(_) => {
                                                        info!(
                                                            "update balance success: {:?}",
                                                            balance
                                                        );
                                                    }
                                                    Err(e) => {
                                                        error!("update balance error: {:?}", e);
                                                    }
                                                }
                                            }
                                            if positions.len() > 0 {
                                                match self
                                                    .db_client
                                                    .update_positions(&positions)
                                                    .await
                                                {
                                                    Ok(_) => {
                                                        info!(
                                                            "update positions success: {:?}",
                                                            positions
                                                        );
                                                    }
                                                    Err(e) => {
                                                        error!("update positions error: {:?}", e);
                                                    }
                                                }
                                            }
                                        } else {
                                            error!("Parse account error: {}", data);
                                        }
                                    }
                                }
                                _ => {}
                            },
                            Err(e) => {
                                error!("Receive message error: {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Connect Error: {:?}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber;
    #[tokio::test]
    async fn test_start_listen() {
        tracing_subscriber::fmt::init();
        let settings_path =
            "/Users/admin/Documents/GitHub/Prajna-Trader/settings/local_settings.yaml";
        let mut listener = OrderListener::default();
        listener.start_listen(settings_path).await;
    }
}
