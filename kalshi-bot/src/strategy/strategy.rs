use async_trait::async_trait;
use protocol::protocol::ServiceName;

#[async_trait]
pub trait Strategy<T> {
    fn name() -> String;
    fn datasources() -> Vec<ServiceName>;
    async fn run(&mut self) -> tokio::io::Result<()>;
}
