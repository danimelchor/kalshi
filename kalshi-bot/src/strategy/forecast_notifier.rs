use crate::strategy::Strategy;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono_tz::Tz;
use protocol::protocol::Event;
use protocol::protocol::MultiServiceSubscriber;
use protocol::protocol::ServiceName;
use std::collections::BTreeMap;
use weather::forecast::fetcher::WeatherForecast;
use weather::temperature::Temperature;

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
    forecast: BTreeMap<DateTime<Tz>, Temperature>,
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
                    let forecast: BTreeMap<DateTime<Tz>, Temperature> = data
                        .message
                        .0
                        .into_iter()
                        .filter(|(k, _)| {
                            let dt: DateTime<Tz> = (*k).into();
                            dt.date_naive() == *date
                        })
                        .map(|(k, v)| (k.into(), v))
                        .collect();

                    self.forecast.extend(forecast);
                    println!("Weather forecast: {:?}", self.forecast);
                }
            })
            .await;

        Ok(())
    }
}
