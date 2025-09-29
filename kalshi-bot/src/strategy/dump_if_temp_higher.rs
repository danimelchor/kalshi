use async_trait::async_trait;
use weather::observations::hourly::NWSHourlyTemperatures;

use crate::strategy::strategy::Strategy;
use protocol::protocol::{Event, MultiServiceSubscriber, ServiceName};

#[derive(Debug)]
pub enum WeatherEvents {
    HourlyWeatherObservation(Event<NWSHourlyTemperatures>),
}

impl From<Event<NWSHourlyTemperatures>> for WeatherEvents {
    fn from(event: Event<NWSHourlyTemperatures>) -> Self {
        WeatherEvents::HourlyWeatherObservation(event)
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
            .listen_all(|event| match event {
                WeatherEvents::HourlyWeatherObservation(data) => {
                    println!("Hourly weather observation: {:?}", data)
                }
            })
            .await;

        Ok(())
    }
}
