use async_trait::async_trait;
use bincode::{Decode, Encode};
use chrono::Utc;

use protocol::{
    datetime::SerializableDateTime,
    protocol::{Event, ServiceName, ServicePublisher},
};

#[derive(Debug, Encode, Decode)]
pub struct DataSourceEvent<T> {
    pub data: T,
    pub is_republished: bool,
    pub ts: SerializableDateTime,
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
pub trait DataSource<T>
where
    T: Encode + Send + Sync,
{
    fn name() -> String;
    fn service_name() -> ServiceName;

    async fn fetch_data(&mut self) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;

    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut publisher = ServicePublisher::new(Self::service_name()).await?;
        let mut event_id = 0u32;

        // Wait for unix socket
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        loop {
            match self.fetch_data().await {
                Ok(data) => {
                    let event = Event::new(event_id, data);
                    if let Err(e) = publisher.publish(&event).await {
                        eprintln!("Failed to publish event for {}: {}", Self::name(), e);
                    }
                    event_id = event_id.wrapping_add(1);
                }
                Err(e) => {
                    eprintln!("Failed to fetch data for {}: {}", Self::name(), e);
                }
            }

            // TODO: adjust sleep time
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}
