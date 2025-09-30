use crate::strategy::strategy::Strategy;
use async_trait::async_trait;
use protocol::protocol::{Event, MultiServiceSubscriber, ServiceName};
use std::sync::{Arc, Mutex};
use weather::forecast::fetcher::WeatherForecast;

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
pub struct ForecastNotifier {
    buffer: Arc<Mutex<Option<WeatherForecast>>>,
}

#[async_trait]
impl Strategy<WeatherEvents> for ForecastNotifier {
    async fn run(&mut self) -> tokio::io::Result<()> {
        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
        client
            .add_subscription::<WeatherForecast>(ServiceName::WeatherForecast)
            .await?;

        let buffer = Arc::clone(&self.buffer);
        let _event_listener = client
            .listen_all(|event| match event {
                WeatherEvents::WeatherForecast(data) => {
                    println!("Weather forecast data: {:?}", data);
                    *buffer.lock().unwrap() = Some(data.message);
                }
            })
            .await;

        Ok(())
    }
}
