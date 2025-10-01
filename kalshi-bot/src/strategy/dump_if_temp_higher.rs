use crate::strategy::strategy::Strategy;
use async_trait::async_trait;
use chrono::NaiveDate;
use protocol::protocol::{Event, MultiServiceSubscriber, ServiceName};
use weather::observations::{
    nws_daily_report::NWSDailyReport, nws_hourly_timeseries::NWSHourlyTimeseriesTemperatures,
};

#[derive(Debug)]
pub enum WeatherEvents {
    HourlyWeatherObservation(Event<NWSHourlyTimeseriesTemperatures>),
    DailyWeatherObservations(Event<NWSDailyReport>),
}

impl From<Event<NWSHourlyTimeseriesTemperatures>> for WeatherEvents {
    fn from(event: Event<NWSHourlyTimeseriesTemperatures>) -> Self {
        WeatherEvents::HourlyWeatherObservation(event)
    }
}

impl From<Event<NWSDailyReport>> for WeatherEvents {
    fn from(event: Event<NWSDailyReport>) -> Self {
        WeatherEvents::DailyWeatherObservations(event)
    }
}

#[derive(Default)]
pub struct DumpIfTempHigher();

#[async_trait]
impl Strategy<WeatherEvents> for DumpIfTempHigher {
    async fn run(&mut self, date: &NaiveDate) -> tokio::io::Result<()> {
        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
        client
            .add_subscription::<NWSHourlyTimeseriesTemperatures>(
                ServiceName::HourlyWeatherTimeseries,
            )
            .await?;
        client
            .add_subscription::<NWSDailyReport>(ServiceName::DailyWeatherReport)
            .await?;

        client
            .listen_all(|event| match event {
                WeatherEvents::HourlyWeatherObservation(data) => {
                    // println!("Hourly weather observation: {:?}", data)
                }
                WeatherEvents::DailyWeatherObservations(data) => {
                    println!("Daily weather observation: {:?}", data)
                }
            })
            .await;

        Ok(())
    }
}
