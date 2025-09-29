use async_trait::async_trait;

#[async_trait]
pub trait Strategy<T> {
    async fn run(&mut self) -> tokio::io::Result<()>;
}
