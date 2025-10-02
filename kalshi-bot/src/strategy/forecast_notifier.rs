use crate::strategy::Strategy;
use anyhow::Result;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono_tz::Tz;
use protocol::protocol::Event;
use protocol::protocol::MultiServiceSubscriber;
use protocol::protocol::ServiceName;
use std::collections::BTreeMap;
use std::sync::Arc;
use telegram::client::TelegramClient;
use tokio::sync::Mutex;
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

pub struct ForecastNotifier {
    forecast: Arc<Mutex<BTreeMap<DateTime<Tz>, Temperature>>>,
    telegram_client: Arc<Mutex<TelegramClient>>,
}

impl ForecastNotifier {
    pub async fn new() -> Self {
        let telegram_client = TelegramClient::start()
            .await
            .expect("Create telegram client");
        Self {
            forecast: Arc::new(Mutex::new(BTreeMap::new())),
            telegram_client: Arc::new(Mutex::new(telegram_client)),
        }
    }
}

async fn handle_event(
    event: WeatherEvents,
    date: &NaiveDate,
    forecast: &Arc<Mutex<BTreeMap<DateTime<Tz>, Temperature>>>,
    telegram_client: &Arc<Mutex<TelegramClient>>,
) -> Result<()> {
    match event {
        WeatherEvents::WeatherForecast(data) => {
            let complete = data.message.complete;
            let new_forecast: BTreeMap<DateTime<Tz>, Temperature> = data
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
                let mut forecast = forecast.lock().await;
                forecast.extend(new_forecast);
                if let Some((dt, max_temp)) = forecast.iter().max_by_key(|(_, v)| *v) {
                    println!("Max temperature {}F at {}", max_temp.as_fahrenheit(), dt);
                    telegram_client
                        .lock()
                        .await
                        .message()
                        .with_title("Forecast update")
                        .with_body(format!(
                            "Max temperature {}F at {}",
                            max_temp.as_fahrenheit(),
                            dt
                        ))
                        .send()
                        .await?;
                }
            } else {
                // TODO: handle partial bayesian updates
                println!(
                    "Partial forecast ({}/{})",
                    data.message.num_lead_times, data.message.total_lead_times,
                );
            }
        }
    }
    Ok(())
}

#[async_trait]
impl Strategy<WeatherEvents> for ForecastNotifier {
    async fn run(&mut self, date: &NaiveDate) -> Result<()> {
        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
        client
            .add_subscription::<WeatherForecast>(ServiceName::WeatherForecast)
            .await?;

        let forecast = self.forecast.clone();
        let telegram_client = self.telegram_client.clone();
        let _event_listener = client
            .listen_all(|event| {
                let forecast = forecast.clone();
                let telegram_client = telegram_client.clone();
                async move {
                    handle_event(event, date, &forecast, &telegram_client).await?;
                    Ok(())
                }
            })
            .await?;

        Ok(())
    }
}
