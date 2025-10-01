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
                    let complete = data.message.complete;
                    let forecast: BTreeMap<DateTime<Tz>, Temperature> = data
                        .message
                        .forecast
                        .into_iter()
                        .filter(|(k, _)| {
                            let dt: DateTime<Tz> = (*k).into();
                            dt.date_naive() == *date
                        })
                        .map(|(k, v)| (k.into(), v))
                        .collect();

                    if complete {
                        self.forecast.extend(forecast);
                        if let Some((dt, max_temp)) = self.forecast.iter().max_by_key(|(_, v)| *v) {
                            println!("Max temperature {}F at {}", max_temp.as_fahrenheit(), dt);
                        }
                    } else {
                        // TODO: handle partial bayesian updates
                        println!(
                            "Partial forecast ({}/{})",
                            data.message.num_lead_times, data.message.num_lead_times
                        );
                    }
                }
            })
            .await;

        Ok(())
    }
}
