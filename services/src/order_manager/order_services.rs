use public::base_model::trade_model::order_model::{Order, OrderResponse};
use public::base_model::error_model::RequestError;
use public::base_model::error_model::StrategyError;
use public::tools::{settings_tools, time_tools, api_tools};
use order_service::order_service_server::{OrderService, OrderServiceServer};
use order_service::{CancelOrderRequest, MakeOrderReply, MakeOrderRequest};
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod order_service {
    tonic::include_proto!("order_service");
}

#[derive(Debug, Clone)]
pub struct GeneralOrderService {
    req_url: String,
    api_key: String,
    secret_key: String,
    client: reqwest::Client,
}

impl Default for GeneralOrderService {
    fn default() -> Self {
        Self {
            req_url: "https://fapi.binance.com".to_owned(),
            api_key: "".to_owned(),
            secret_key: "".to_owned(),
            client: reqwest::Client::new(),
        }
    }
}

#[tonic::async_trait]
impl OrderService for GeneralOrderService {
    async fn make_order(
        &self,
        request: Request<MakeOrderRequest>,
    ) -> Result<Response<MakeOrderReply>, Status> {
        let req = request.into_inner();
        let cur_req = MakeOrderRequest {
            symbol: req.symbol.clone(),
            price: req.price,
            quantity: req.quantity,
            side: req.side.clone(),
            strategy: req.strategy.clone(),
        };

        match self.create_order(&cur_req).await {
            Ok(order) => {
                let reply = MakeOrderReply {
                    symbol: order.get_symbol().into(),
                    price: order.get_price(),
                    quantity: order.get_qty(),
                    side: order.get_side().string(),
                    strategy: order.get_strategy_name(),
                    status: order.get_status().string(),
                    order_cid: order.get_cid().into(),
                };
                return Ok(tonic::Response::new(reply));
            }
            Err(e) => {
                return Err(Status::new(tonic::Code::Internal, format!("{:?}", e)));
            }
        };
    }

    async fn stop_loss_order(
        &self,
        request: Request<MakeOrderRequest>,
    ) -> Result<Response<MakeOrderReply>, Status> {
        let req = request.into_inner();
        let cur_req = MakeOrderRequest {
            symbol: req.symbol.clone(),
            price: req.price,
            quantity: req.quantity,
            side: req.side.clone(),
            strategy: req.strategy.clone(),
        };

        match self.make_stop_loss_order(&cur_req).await {
            Ok(order) => {
                let reply = MakeOrderReply {
                    symbol: order.get_symbol().into(),
                    price: order.get_price(),
                    quantity: order.get_qty(),
                    side: order.get_side().string(),
                    strategy: order.get_strategy_name(),
                    status: order.get_status().string(),
                    order_cid: order.get_cid().into(),
                };
                return Ok(tonic::Response::new(reply));
            }
            Err(e) => {
                return Err(Status::new(tonic::Code::Internal, format!("{:?}", e)));
            }
        };
    }

    async fn take_profit_order(
        &self,
        request: Request<MakeOrderRequest>,
    ) -> Result<Response<MakeOrderReply>, Status> {
        let req = request.into_inner();
        let cur_req = MakeOrderRequest {
            symbol: req.symbol.clone(),
            price: req.price,
            quantity: req.quantity,
            side: req.side.clone(),
            strategy: req.strategy.clone(),
        };

        match self.make_take_profit_order(&cur_req).await {
            Ok(order) => {
                let reply = MakeOrderReply {
                    symbol: order.get_symbol().into(),
                    price: order.get_price(),
                    quantity: order.get_qty(),
                    side: order.get_side().string(),
                    strategy: order.get_strategy_name(),
                    status: order.get_status().string(),
                    order_cid: order.get_cid().into(),
                };
                return Ok(tonic::Response::new(reply));
            }
            Err(e) => {
                return Err(Status::new(tonic::Code::Internal, format!("{:?}", e)));
            }
        };
    }

    async fn cancel_order(
        &self,
        request: Request<CancelOrderRequest>,
    ) -> Result<Response<MakeOrderReply>, Status> {
        let req = request.into_inner();

        match self.make_cancel_order(&req).await {
            Ok(order) => {
                let reply = MakeOrderReply {
                    symbol: order.get_symbol().into(),
                    price: order.get_price(),
                    quantity: order.get_qty(),
                    side: order.get_side().string(),
                    strategy: order.get_strategy_name(),
                    status: order.get_status().string(),
                    order_cid: order.get_cid().into(),
                };
                return Ok(tonic::Response::new(reply));
            }
            Err(e) => {
                return Err(Status::new(tonic::Code::Internal, format!("{:?}", e)));
            }
        };
    }
}

impl GeneralOrderService {
    pub fn load_settings(&mut self, path: &str) {
        let settings = settings_tools::load_settings(path);
        self.api_key = settings.get_api_key();
        self.secret_key = settings.get_secret_key();
    }

    fn parse_response_order(&self, data: String) -> Option<Order> {
        if let Ok(order_reponse) = serde_json::from_str::<OrderResponse>(&data) {
            Some(order_reponse.order_response_into_order())
        } else {
            None
        }
    }

    fn parse_response_error(&self, data: String) -> Option<RequestError> {
        if let Ok(request_error) = serde_json::from_str::<RequestError>(&data) {
            Some(request_error)
        } else {
            None
        }
    }

    fn convert_base_order_request_into_params(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
    ) -> String {
        format!(
            "symbol={}&side={}&quantity={}&timestamp={}&timeInForce=GTC&recvWindow=5000",
            symbol,
            side,
            quantity,
            time_tools::get_now_timestamp()
        )
    }

    fn convert_cancel_order_request_into_params(&self, symbol: &str, cid: &str) -> String {
        format!(
            "symbol={}&origClientOrderId={}&timestamp={}&recvWindow=5000",
            symbol,
            cid,
            time_tools::get_now_timestamp()
        )
    }

    fn convert_make_order_request_into_params(
        &self,
        symbol: &str,
        side: &str,
        price: f64,
        quantity: f64,
        strategy: &str,
    ) -> String {
        let mut params = self.convert_base_order_request_into_params(symbol, side, quantity);
        let cid = self.generate_cid(symbol, strategy);
        params.push_str(&format!(
            "&newClientOrderId={}&price={}&type=LIMIT",
            cid, price
        ));
        params
    }

    fn conver_stop_loss_order_request_into_params(
        &self,
        symbol: &str,
        side: &str,
        price: f64,
        quantity: f64,
        strategy: &str,
    ) -> String {
        let mut params = self.convert_base_order_request_into_params(symbol, side, quantity);
        let cid = self.generate_cid(symbol, strategy);
        params.push_str(&format!(
            "&newClientOrderId={}&stopPrice={}&type=STOP_MARKET",
            cid, price
        ));
        params
    }

    fn conver_take_profit_order_request_into_params(
        &self,
        symbol: &str,
        side: &str,
        price: f64,
        quantity: f64,
        strategy: &str,
    ) -> String {
        let mut params = self.convert_base_order_request_into_params(symbol, side, quantity);
        let cid = self.generate_cid(symbol, strategy);
        params.push_str(&format!(
            "&newClientOrderId={}&stopPrice={}&type=TAKE_PROFIT_MARKET",
            cid, price
        ));
        params
    }

    fn generate_cid(&self, symbol: &str, strategy: &str) -> String {
        let timestamp = time_tools::get_now_timestamp();
        format!("{}_{}_{}", strategy, symbol, timestamp)
    }

    async fn base_create_order(&self, params: &str) -> Result<Order, StrategyError> {
        let signature = api_tools::get_signature(&self.secret_key, &params);
        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
            self.req_url, params, signature
        );
        match self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", self.api_key.clone())
            .send()
            .await
        {
            Ok(res) => match res.text().await {
                Ok(text) => {
                    if let Some(order) = self.parse_response_order(text.clone()) {
                        println!("{:?}", order);
                        return Ok(order);
                    } else {
                        if let Some(request_error) = self.parse_response_error(text.clone()) {
                            return Err(request_error.parse_request_error_into_strategy_error());
                        } else {
                            return Err(StrategyError::PlaceOrderError(
                                "Create order parse failed".to_string(),
                            ));
                        }
                    }
                }
                Err(e) => {
                    return Err(StrategyError::PlaceOrderError(format!(
                        "Create order failed: {}",
                        e
                    )));
                }
            },
            Err(e) => {
                return Err(StrategyError::PlaceOrderError(format!(
                    "Create order failed: {}",
                    e
                )));
            }
        }
    }

    async fn create_order(&self, order: &MakeOrderRequest) -> Result<Order, StrategyError> {
        let params = self.convert_make_order_request_into_params(
            &order.symbol,
            &order.side,
            order.price,
            order.quantity,
            &order.strategy,
        );
        self.base_create_order(&params).await
    }

    async fn make_take_profit_order(
        &self,
        order: &MakeOrderRequest,
    ) -> Result<Order, StrategyError> {
        let params = self.conver_take_profit_order_request_into_params(
            &order.symbol,
            &order.side,
            order.price,
            order.quantity,
            &order.strategy,
        );
        self.base_create_order(&params).await
    }

    async fn make_stop_loss_order(&self, order: &MakeOrderRequest) -> Result<Order, StrategyError> {
        let params = self.conver_stop_loss_order_request_into_params(
            &order.symbol,
            &order.side,
            order.price,
            order.quantity,
            &order.strategy,
        );
        self.base_create_order(&params).await
    }

    async fn make_cancel_order(&self, order: &CancelOrderRequest) -> Result<Order, StrategyError> {
        let params = self.convert_cancel_order_request_into_params(
            &order.symbol.clone(),
            &order.order_cid.clone(),
        );
        let signature = api_tools::get_signature(&self.secret_key, &params);
        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
            self.req_url, params, signature
        );
        match self
            .client
            .delete(&url)
            .header("X-MBX-APIKEY", self.api_key.clone())
            .send()
            .await
        {
            Ok(res) => match res.text().await {
                Ok(text) => {
                    if let Some(order) = self.parse_response_order(text.clone()) {
                        println!("{:?}", order);
                        return Ok(order);
                    } else {
                        if let Some(request_error) = self.parse_response_error(text.clone()) {
                            return Err(request_error.parse_request_error_into_strategy_error());
                        } else {
                            return Err(StrategyError::PlaceOrderError(
                                "Cancel order parse failed".to_string(),
                            ));
                        }
                    }
                }
                Err(e) => {
                    println!("get error: {}", e);
                    return Err(StrategyError::PlaceOrderError(format!(
                        "Cancel order failed: {}",
                        e
                    )));
                }
            },
            Err(e) => {
                println!("get error: {}", e);
                return Err(StrategyError::PlaceOrderError(format!(
                    "Cancel order failed: {}",
                    e
                )));
            }
        }
    }

    pub async fn start_order_service(&mut self, path: &str) {
        let addr = "[::1]:50051".parse().unwrap();
        self.load_settings(path);
        info!("start order service...");
        Server::builder()
            .add_service(OrderServiceServer::new(self.clone()))
            .serve(addr)
            .await
            .unwrap();
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber;
    #[tokio::test]
    async fn test_order_service() {
        tracing_subscriber::fmt::init();
        let settings_path =
            "/Users/admin/Documents/GitHub/Prajna-Trader/settings/local_settings.yaml";
        // start_order_service(&settings_path).await;
        let mut service = GeneralOrderService::default();
        service.start_order_service(&settings_path).await;
    }
}
