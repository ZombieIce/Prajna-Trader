use crate::market_data_module::general_data;
use mongodb::{error::Error, Client, Collection};

pub struct MongoEngine {
    url: String,
}

impl MongoEngine {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    pub async fn get_client(&self) -> Result<Client, Error> {
        Client::with_uri_str(&self.url).await
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
                    let result = collection.insert_one(kline[0].clone(), None).await?;
                    println!("Inserted document with id: {:?}", result.inserted_id);
                    Ok(())
                } else {
                    let result = collection.insert_many(kline, None).await?;
                    println!("Inserted {} documents", result.inserted_ids.len());
                    Ok(())
                }
            }
            Err(e) => Err(e),
        }
    }
}
