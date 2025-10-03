use anyhow::Result;
use kalshi_api_spec::{
    event::EventResponse,
    market::MarketResponse,
    ticker::{EventTicker, MarketTicker},
};

use crate::{
    client::{BaseUrl, KalshiApiClient, SafeSend},
    keys::{ApiKey, PrivateKey},
};

pub struct MarketsApiClient(KalshiApiClient);

impl MarketsApiClient {
    pub fn new(api_key: ApiKey, private_key: PrivateKey, base_url: BaseUrl) -> Self {
        let client = KalshiApiClient::new(api_key, private_key, base_url);
        Self(client)
    }

    pub async fn get_event(&self, ticker: &EventTicker) -> Result<EventResponse> {
        self.0.get(&format!("/events/{ticker}"))?.safe_send().await
    }

    pub async fn get_market(&self, ticker: &MarketTicker) -> Result<MarketResponse> {
        self.0.get(&format!("/markets/{ticker}"))?.safe_send().await
    }
}
