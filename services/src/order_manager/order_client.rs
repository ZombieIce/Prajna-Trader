use order_service::order_service_client::OrderServiceClient;
use order_service::{CancelOrderRequest, MakeOrderRequest};
use tonic::transport::Channel;

pub mod order_service {
    tonic::include_proto!("order_service");
}

#[derive(Debug)]
pub struct GeneralOrderClient {
    server_url: String,
    client: Option<OrderServiceClient<Channel>>,
}

impl Default for GeneralOrderClient {
    fn default() -> Self {
        Self {
            server_url: "http://[::1]:50051".to_owned(),
            client: None,
        }
    }
}

impl GeneralOrderClient {
    pub async fn connect(&mut self) {
        self.client = Some(
            OrderServiceClient::connect(format!("{}", self.server_url))
                .await
                .unwrap(),
        );
    }

    pub async fn make_order(
        &mut self,
        symbol: String,
        price: f64,
        quantity: f64,
        side: String,
        strategy: String,
    ) {
        let request = tonic::Request::new(MakeOrderRequest {
            symbol,
            side,
            price,
            quantity,
            strategy,
        });
        let mut client = self.client.clone().unwrap();
        match client.make_order(request).await {
            Ok(response) => {
                println!("RESPONSE={:?}", response);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    pub async fn cancel_order(
        &mut self,
        symbol: String,
        order_cid: String,
    ) {
        let request = tonic::Request::new(CancelOrderRequest {
            symbol,
            order_cid,
        });
        let mut client = self.client.clone().unwrap();
        match client.cancel_order(request).await {
            Ok(response) => {
                println!("RESPONSE={:?}", response);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_create_order() {
        let mut client = GeneralOrderClient::default();
        client.connect().await;
        client
            .make_order("BTCUSDT".to_string(), 67400.0, 0.002, "SELL".to_string(), "test".to_string())
            .await;
    }

    #[tokio::test]
    async fn test_cancel_order() {
        let mut client = GeneralOrderClient::default();
        client.connect().await;
        client
            .cancel_order("BTCUSDT".to_string(), "test_BTCUSDT_1721988794300".to_string())
            .await;
    }
}


