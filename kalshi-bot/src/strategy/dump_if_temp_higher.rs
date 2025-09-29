use async_trait::async_trait;
use weather::observations::{daily::NWSDailyReport, hourly::NWSHourlyTemperatures};

use crate::strategy::strategy::Strategy;
use protocol::protocol::{Event, MultiServiceSubscriber, ServiceName};

#[derive(Debug)]
pub enum WeatherEvents {
    HourlyWeatherObservation(Event<NWSHourlyTemperatures>),
    DailyWeatherObservations(Event<NWSDailyReport>),
}

impl From<Event<NWSHourlyTemperatures>> for WeatherEvents {
    fn from(event: Event<NWSHourlyTemperatures>) -> Self {
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
    async fn run(&mut self) -> tokio::io::Result<()> {
        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
        client
            .add_subscription::<NWSHourlyTemperatures>(ServiceName::HourlyWeatherObservations)
            .await?;
        client
            .add_subscription::<NWSDailyReport>(ServiceName::DailyWeatherObservations)
            .await?;

        client
            .listen_all(|event| match event {
                WeatherEvents::HourlyWeatherObservation(data) => {
                    println!("Hourly weather observation: {:?}", data)
                }
                WeatherEvents::DailyWeatherObservations(data) => {
                    println!("Daily weather observation: {:?}", data)
                }
            })
            .await;

        Ok(())
    }
}
