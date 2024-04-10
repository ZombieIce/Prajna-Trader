use crate::market_data_module::general_data;
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::doc,
    error::Error,
    options::{FindOneOptions, FindOptions},
    Client, Collection,
};

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

    pub async fn update_exchange_info(
        &self,
        exchange_info: &general_data::ExchangeInfo,
    ) -> Result<(), Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database(&self.database);
                let collection: Collection<general_data::ExchangeInfo> =
                    db.collection("exchange_info");
                let filter = doc! {"$and": [
                    {"exchange": exchange_info.get_exchange().clone()},
                    {"market_type": exchange_info.get_market_type().clone()}

                ]};
                collection.delete_one(filter, None).await?;
                collection.insert_one(exchange_info.clone(), None).await?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_exchange_info(&self) -> Option<general_data::ExchangeInfo> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database(&self.database);
                let collection: Collection<general_data::ExchangeInfo> =
                    db.collection("exchange_info");
                let result = collection.find_one(None, None).await.unwrap();
                match result {
                    Some(exchange_info) => Some(exchange_info),
                    None => None,
                }
            }
            Err(_) => None,
        }
    }

    pub async fn insert_kline(
        &self,
        symbol: &str,
        kline: &Vec<general_data::Kline>,
    ) -> Result<(), Error> {
        if kline.len() == 0 {
            return Ok(());
        }
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("klines");
                let collection: Collection<general_data::Kline> = db.collection(symbol);
                if kline.len() == 1 {
                    match collection.insert_one(kline[0].clone(), None).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    }
                } else {
                    match collection.insert_many(kline.clone(), None).await {
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
    ) -> Result<Option<Vec<general_data::Kline>>, Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("klines");
                let collections = db.list_collection_names(None).await?;
                if collections.contains(&symbol.to_string()) {
                    let collection: Collection<general_data::Kline> = db.collection(symbol);

                    let filter = doc! { "open_time": { "$gte": start_date } };
                    let options = FindOptions::builder().sort(doc! { "open_time": 1 }).build();
                    match collection.find(filter, options).await {
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

    pub async fn fetch_latest_kline(
        &self,
        symbol: &str,
    ) -> Result<Option<general_data::Kline>, Error> {
        match self.get_client().await {
            Ok(client) => {
                let db = client.database("klines");
                let collections = db.list_collection_names(None).await?;
                if collections.contains(&symbol.to_string()) {
                    let collection: Collection<general_data::Kline> = db.collection(symbol);

                    let options = FindOneOptions::builder()
                        .sort(doc! { "open_time": -1 })
                        .build();

                    match collection.find_one(None, options).await {
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
}
