use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
pub trait Strategy<T> {
    async fn run(&mut self, date: &NaiveDate) -> Result<()>;
}
