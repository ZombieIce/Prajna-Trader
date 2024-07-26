use futures_util::stream::TryStreamExt;
// use futures::TryStreamExt;
use mongodb::{bson::doc, error::Error, Client, Collection};
use public::base_model::info_model::ExchangeInfo;
use public::base_model::market_model::kline_model::Kline;
use public::base_model::trade_model::order_model::Order;
use public::base_model::trade_model::position_model::Position;
use public::strategy_model::strategy_portfolio::Balance;

#[derive(Clone)]
pub struct MongoEngine {
    url: String,
    database: String,
}

impl Default for MongoEngine {
    fn default() -> Self {
        Self {
            url: "mongodb://localhost:27017".to_string(),
            database: "prajna".to_string(),
        }
    }
}

impl MongoEngine {
    pub fn new(url: &str, database: &str) -> Self {
        Self {
            url: url.to_string(),
            database: database.to_string(),
        }
    }

    pub async fn get_client(&self) -> Result<Client, Error> {
        Client::with_uri_str(&self.url).await
    }

    pub async fn update_exchange_info(&self, exchange_info: &ExchangeInfo) -> Result<(), Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database(&self.database);
                let collection: Collection<ExchangeInfo> = db.collection("exchange_info");
                let filter = doc! {"$and": [
                    {"exchange": exchange_info.get_exchange().clone()},
                    {"market_type": exchange_info.get_market_type().clone()}

                ]};
                collection.delete_one(filter).await?;
                collection.insert_one(exchange_info.clone()).await?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_exchange_info(&self) -> Option<ExchangeInfo> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database(&self.database);
                let collection: Collection<ExchangeInfo> = db.collection("exchange_info");
                match collection.find_one(doc! {}).await {
                    Ok(result) => match result {
                        Some(exchange_info) => Some(exchange_info),
                        None => None,
                    },
                    Err(e) => {
                        println!("get_exchange Error: {}", e);
                        None
                    }
                }
            }
            Err(_) => None,
        }
    }

    pub async fn insert_kline(&self, symbol: &str, kline: &Vec<Kline>) -> Result<(), Error> {
        if kline.len() == 0 {
            return Ok(());
        }
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("klines");
                let collection: Collection<Kline> = db.collection(symbol);
                if kline.len() == 1 {
                    match collection.insert_one(kline[0].clone()).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    }
                } else {
                    match collection.insert_many(kline.clone()).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn fetch_klines(
        &self,
        symbol: &str,
        start_date: i64,
    ) -> Result<Option<Vec<Kline>>, Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("klines");
                let collections = db.list_collection_names().await?;
                if collections.contains(&symbol.to_string()) {
                    let collection: Collection<Kline> = db.collection(symbol);
                    let filter = doc! { "open_time": { "$gte": start_date } };
                    match collection.find(filter).sort(doc! {"open_time": 1}).await {
                        Ok(cursor) => match cursor.try_collect().await {
                            Ok(res) => Ok(Some(res)),
                            Err(e) => Err(e),
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    Err(Error::from(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Collection not found",
                    )))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn fetch_latest_kline(&self, symbol: &str) -> Result<Option<Kline>, Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("klines");
                let collections = db.list_collection_names().await?;
                if collections.contains(&symbol.to_string()) {
                    let collection: Collection<Kline> = db.collection(symbol);
                    match collection
                        .find_one(doc! {})
                        .sort(doc! {"open_time": -1})
                        .await
                    {
                        Ok(result) => match result {
                            Some(kline) => Ok(Some(kline)),
                            None => Ok(None),
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn insert_order(&self, order: &Order) -> Result<(), Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("orders");
                let collection: Collection<Order> = db.collection(order.get_symbol());
                match collection.insert_one(order).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn fetch_orders(&self, symbol: &str) -> Result<Option<Vec<Order>>, Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("orders");
                let collections = db.list_collection_names().await?;
                if collections.contains(&symbol.to_string()) {
                    let collection: Collection<Order> = db.collection(symbol);
                    match collection.find(doc! {}).sort(doc! {"timestamp": 1}).await {
                        Ok(cursor) => match cursor.try_collect().await {
                            Ok(res) => Ok(Some(res)),
                            Err(e) => Err(e),
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn update_positions(&self, positions: &Vec<Position>) -> Result<(), Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("balance");
                let collections = db.list_collection_names().await?;
                let collection: Collection<Position> = db.collection("positions");
                if collections.contains(&"positions".to_string()) {
                    match collection.delete_many(doc! {}).await {
                        Ok(_) => match collection.insert_many(positions.clone()).await {
                            Ok(_) => Ok(()),
                            Err(e) => Err(e),
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    match collection.insert_many(positions.clone()).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_position(&self, symbol: &str) -> Result<Option<Position>, Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("balance");
                let collections = db.list_collection_names().await?;
                if collections.contains(&"positions".to_string()) {
                    let collection: Collection<Position> = db.collection("positions");
                    match collection.find_one(doc! {"symbol": symbol}).await {
                        Ok(result) => match result {
                            Some(position) => Ok(Some(position)),
                            None => Ok(None),
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn update_balance(&self, balance: &Balance) -> Result<(), Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("balance");
                let collection: Collection<Balance> = db.collection("balance");
                match collection.delete_many(doc! {}).await {
                    Ok(_) => match collection.insert_one(balance.clone()).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_balance(&self) -> Result<Option<Balance>, Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("balance");
                let collection: Collection<Balance> = db.collection("balance");
                match collection.find_one(doc! {}).await {
                    Ok(result) => match result {
                        Some(balance) => Ok(Some(balance)),
                        None => Ok(None),
                    },
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_fetch_latest_kline() {
        let client = MongoEngine::default();
        match client.fetch_latest_kline("btcusdt").await {
            Ok(res) => {
                if let Some(k) = res {
                    println!("{:?}", k);
                }
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
    }
}
