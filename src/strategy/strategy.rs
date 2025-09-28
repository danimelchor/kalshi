use async_trait::async_trait;

use crate::protocol::protocol::{ServiceName, ServiceSubscriber};

#[async_trait]
pub trait Strategy<T> {
    fn name(&self) -> &str;
    fn datasources(&self) -> &Vec<ServiceName>;

    async fn handle_event(&self);

    async fn run(&mut self) {
        let subscriber = ServiceSubscriber::new();
        for datasource in self.datasources() {
            subscriber.subscribe(datasource).await?
        }
    }
}
