use crate::strategy::strategy::Strategy;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;
use protocol::protocol::{Event, MultiServiceSubscriber, ServiceName};
use weather::observations::{
    nws_daily_report::NWSDailyReport, nws_hourly_table::NWSHourlyTableTemperatures,
    nws_hourly_timeseries::NWSHourlyTimeseriesTemperatures,
};

#[derive(Debug)]
pub enum WeatherEvents {
    HourlyWeatherTimeseries(NWSHourlyTimeseriesTemperatures),
    HourlyWeatherTable(NWSHourlyTableTemperatures),
    DailyWeatherReport(NWSDailyReport),
}

impl From<Event<NWSHourlyTimeseriesTemperatures>> for WeatherEvents {
    fn from(event: Event<NWSHourlyTimeseriesTemperatures>) -> Self {
        WeatherEvents::HourlyWeatherTimeseries(event.message)
    }
}

impl From<Event<NWSHourlyTableTemperatures>> for WeatherEvents {
    fn from(event: Event<NWSHourlyTableTemperatures>) -> Self {
        WeatherEvents::HourlyWeatherTable(event.message)
    }
}

impl From<Event<NWSDailyReport>> for WeatherEvents {
    fn from(event: Event<NWSDailyReport>) -> Self {
        WeatherEvents::DailyWeatherReport(event.message)
    }
}

#[derive(Default)]
pub struct DumpIfTempHigher();

#[async_trait]
impl Strategy<WeatherEvents> for DumpIfTempHigher {
    async fn run(&mut self, date: &NaiveDate) -> Result<()> {
        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
        client
            .add_subscription::<NWSHourlyTimeseriesTemperatures>(
                ServiceName::HourlyWeatherTimeseries,
            )
            .await?;
        client
            .add_subscription::<NWSHourlyTableTemperatures>(ServiceName::HourlyWeatherTable)
            .await?;
        client
            .add_subscription::<NWSDailyReport>(ServiceName::DailyWeatherReport)
            .await?;

        client
            .listen_all(|event| async move {
                match event {
                    WeatherEvents::HourlyWeatherTimeseries(data) => {
                        println!("Hourly weather timeseries: {:?}", data.0.last().unwrap())
                    }
                    WeatherEvents::HourlyWeatherTable(data) => {
                        println!("Hourly weather table: {:?}", data.0.last().unwrap())
                    }
                    WeatherEvents::DailyWeatherReport(data) => {
                        println!("Daily weather report: {:?}", data)
                    }
                }
                Ok(())
            })
            .await?;

        Ok(())
    }
}
