use crate::datasource::datasource::DataSource;
use anyhow::Result;
use async_stream::stream;
use futures::Stream;
use protocol::protocol::ServiceName;
use std::time::Duration;
use tokio::time::sleep;
use weather::{
    observations::daily::{NWSDailyObservationFetcher, NWSDailyReport},
    station::Station,
};

pub struct DailyWeatherObservationDataSource {
    fetcher: NWSDailyObservationFetcher,
}

impl DailyWeatherObservationDataSource {
    pub fn new(station: Station) -> Self {
        let fetcher = NWSDailyObservationFetcher::new(station, None);
        Self { fetcher }
    }
}

impl DataSource<NWSDailyReport> for DailyWeatherObservationDataSource {
    fn name() -> String {
        "Weather Forecast".into()
    }

    fn service_name() -> ServiceName {
        ServiceName::DailyWeatherObservations
    }

    fn fetch_data(&mut self) -> impl Stream<Item = Result<NWSDailyReport>> + Send {
        stream! {
            loop {
                let result = self.fetcher
                    .fetch(1, true)
                    .await;
                yield result;
                    sleep(Duration::from_secs(60)).await;
            }
        }
    }
}
