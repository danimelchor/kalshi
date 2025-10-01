use crate::strategy::Strategy;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono_tz::Tz;
use protocol::protocol::Event;
use protocol::protocol::MultiServiceSubscriber;
use protocol::protocol::ServiceName;
use weather::forecast::fetcher::{TemperatureAtTime, WeatherForecast};

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
    max_temperature: Option<TemperatureAtTime>,
}

#[async_trait]
impl Strategy<WeatherEvents> for ForecastNotifier {
    async fn run(&mut self, date: &NaiveDate) -> tokio::io::Result<()> {
        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
        client
            .add_subscription::<WeatherForecast>(ServiceName::WeatherForecast)
            .await?;

        let _event_listener = client
            .listen_all(|event| match event {
                WeatherEvents::WeatherForecast(data) => {
                    if let Some(max_temp) = data
                        .message
                        .temperatures_at_times
                        .into_iter()
                        .filter(|t| {
                            let dt: DateTime<Tz> = t.timestamp.clone().into();
                            dt.date_naive() == *date
                        })
                        .max_by(|t1, t2| t1.temperature.partial_cmp(&t2.temperature).unwrap())
                    {
                        self.max_temperature = Some(max_temp);
                        println!(
                            "Weather forecast max temperature: {:?}",
                            self.max_temperature
                        );
                    }
                }
            })
            .await;

        Ok(())
    }
}
