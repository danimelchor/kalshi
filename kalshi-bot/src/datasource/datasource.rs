use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use futures::{Stream, StreamExt};
use protocol::{
    datetime::DateTimeZoned,
    protocol::{Event, ServiceName, ServicePublisher},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    pin::pin,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSourceEvent<T> {
    pub data: T,
    pub is_republished: bool,
    pub ts: DateTimeZoned,
}

impl<T> DataSourceEvent<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            is_republished: false,
            ts: Utc::now().into(),
        }
    }

    pub fn republished(mut self) -> Self {
        self.is_republished = true;
        self
    }
}

#[async_trait]
pub trait DataSource<T>: Display
where
    T: Serialize + Send + Sync,
{
    fn service_name() -> ServiceName;

    fn fetch_data(&mut self) -> impl Stream<Item = Result<T>> + Send;

    async fn run(&mut self) -> Result<()> {
        let mut publisher = ServicePublisher::new(Self::service_name()).await?;
        let mut event_id = 0u32;

        // Wait for unix socket
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let id = self.to_string();
        let mut stream = pin!(self.fetch_data());
        while let Some(event) = stream.next().await {
            match event {
                Ok(data) => {
                    let event = Event::new(event_id, data);
                    if let Err(e) = publisher.publish(&event).await {
                        eprintln!("Failed to publish event for {}: {:?}", id, e);
                    }
                    event_id = event_id.wrapping_add(1);
                }
                Err(e) => {
                    eprintln!("Failed to fetch data for {}: {:?}", id, e);
                }
            }
        }

        Ok(())
    }
}
