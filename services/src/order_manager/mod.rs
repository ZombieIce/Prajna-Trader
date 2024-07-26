pub mod order_client;
mod order_listener;
mod order_services;

pub struct OrderManager {
    order_listener: order_listener::OrderListener,
    order_services: order_services::GeneralOrderService,
}

impl OrderManager {
    pub fn new() -> Self {
        Self {
            order_listener: order_listener::OrderListener::default(),
            order_services: order_services::GeneralOrderService::default(),
        }
    }

    pub async fn start_service(&mut self, path: &str) {
        let listener = self.order_listener.start_listen(path);
        let service = self.order_services.start_order_service(path);
        tokio::join!(listener, service);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber;
    #[tokio::test]
    async fn test_service() {
        tracing_subscriber::fmt::init();
        let mut order_manager = OrderManager::new();
        let setting_path =
            "/Users/admin/Documents/GitHub/Prajna-Trader/settings/local_settings.yaml";
        order_manager.start_service(setting_path).await;
    }
}
