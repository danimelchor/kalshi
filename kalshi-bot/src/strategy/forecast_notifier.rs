use async_trait::async_trait;

use crate::{
    datasource::{
        datasource::DataSource,
        weather_forecast::{WeatherForecast, WeatherForecastDataSource},
    },
    strategy::strategy::Strategy,
};
use protocol::protocol::{Event, MultiServiceSubscriber, ServiceName};

#[derive(Debug)]
pub enum WeatherEvents {
    WeatherForecast(Event<WeatherForecast>),
}

impl From<Event<WeatherForecast>> for WeatherEvents {
    fn from(event: Event<WeatherForecast>) -> Self {
        WeatherEvents::WeatherForecast(event)
    }
}

#[derive(Default)]
pub struct ExampleStrategy();

#[async_trait]
impl Strategy<WeatherEvents> for ExampleStrategy {
    fn name() -> String {
        "example".into()
    }

    fn datasources() -> Vec<ServiceName> {
        vec![WeatherForecastDataSource::service_name()]
    }

    async fn run(&mut self) -> tokio::io::Result<()> {
        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
        client
            .add_subscription::<WeatherForecast>(ServiceName::WeatherForecast)
            .await?;

        client
            .listen_all(|event| match event {
                WeatherEvents::WeatherForecast(data) => println!("Example.com data: {:?}", data),
            })
            .await;

        Ok(())
    }
}
