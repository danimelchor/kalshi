use anyhow::Result;
use async_trait::async_trait;
use chrono::Local;
use reqwest::{
    Client, Method, RequestBuilder,
    header::{CONTENT_TYPE, HeaderMap},
};
use serde::de::DeserializeOwned;

use crate::keys::{ApiKey, PrivateKey};

pub enum BaseUrl {
    Prod,
    Demo,
}

impl BaseUrl {
    fn url(&self) -> String {
        match self {
            BaseUrl::Prod => "https://api.elections.kalshi.com/trade-api/v2".into(),
            BaseUrl::Demo => "https://demo-api.kalshi.co/trade-api/v2".into(),
        }
    }
}

pub struct KalshiApiClient {
    client: Client,
    api_key: ApiKey,
    private_key: PrivateKey,
    base_url: BaseUrl,
}

impl KalshiApiClient {
    pub fn new(api_key: ApiKey, private_key: PrivateKey, base_url: BaseUrl) -> Self {
        let client = Client::new();
        Self {
            api_key,
            private_key,
            client,
            base_url,
        }
    }

    fn headers(&self, method: &Method, path: &str) -> Result<HeaderMap> {
        let now = Local::now().timestamp_millis();
        let msg = format!("{now}{method}{path}");
        let signed_msg = self.private_key.sign(&msg)?;

        let mut headers = HeaderMap::new();
        let _ = headers.insert(CONTENT_TYPE, "application/json".parse()?);
        let _ = headers.insert("KALSHI-ACCESS-KEY", self.api_key.to_string().parse()?);
        let _ = headers.insert("KALSHI-ACCESS-SIGNATURE", signed_msg.parse()?);
        let _ = headers.insert("KALSHI-ACCESS-TIMESTAMP", now.to_string().parse()?);
        Ok(headers)
    }

    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url = format!("{}{}", self.base_url.url(), path);
        let headers = self.headers(&method, path)?;
        let request = self.client.request(method, url).headers(headers);
        Ok(request)
    }

    pub fn get(&self, path: &str) -> Result<RequestBuilder> {
        self.request(Method::GET, path)
    }

    pub fn post(&self, path: &str) -> Result<RequestBuilder> {
        self.request(Method::POST, path)
    }
}

#[async_trait]
pub trait SafeSend {
    async fn safe_send<T: DeserializeOwned>(self) -> Result<T>;
}

#[async_trait]
impl SafeSend for RequestBuilder {
    async fn safe_send<T: DeserializeOwned>(self) -> Result<T> {
        let result: T = self.send().await?.error_for_status()?.json().await?;
        Ok(result)
    }
}
