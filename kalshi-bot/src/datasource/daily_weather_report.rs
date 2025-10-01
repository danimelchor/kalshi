use crate::datasource::datasource::DataSource;
use anyhow::Result;
use async_stream::stream;
use futures::Stream;
use protocol::protocol::ServiceName;
use std::time::Duration;
use tokio::time::sleep;
use weather::{
    observations::nws_daily_report::{NWSDailyReport, NWSDailyReportFetcher},
    station::Station,
};

pub struct DailyWeatherReportSource {
    fetcher: NWSDailyReportFetcher,
}

impl DailyWeatherReportSource {
    pub fn new(station: Station) -> Self {
        let fetcher = NWSDailyReportFetcher::new(station, None);
        Self { fetcher }
    }
}

impl DataSource<NWSDailyReport> for DailyWeatherReportSource {
    fn name() -> String {
        "Weather Forecast".into()
    }

    fn service_name() -> ServiceName {
        ServiceName::DailyWeatherReport
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
