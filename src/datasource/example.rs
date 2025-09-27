use bincode::{self, Decode, Encode};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{datasource::datasource::DataSource, protocol::protocol::ServiceName};

// Example.com data structure
#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone)]
pub struct ExampleComData {
    pub content: String,
    pub status_code: u16,
    pub content_length: Option<usize>,
}

// Implementation that fetches example.com
pub struct ExampleComDataSource {
    client: reqwest::Client,
}

impl ExampleComDataSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl DataSource<ExampleComData> for ExampleComDataSource {
    fn name(&self) -> &str {
        "Example.com Fetcher"
    }

    fn service_name(&self) -> ServiceName {
        ServiceName::GovForecast
    }

    async fn fetch_data(&mut self) -> Result<ExampleComData, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.get("http://example.com").send().await?;
        let status_code = response.status().as_u16();
        let content_length = response.content_length().map(|l| l as usize);
        let content = response.text().await?;

        Ok(ExampleComData {
            content,
            status_code,
            content_length,
        })
    }
}
