use crate::strategy::Strategy;
use crate::strategy::utils::check_dates_match;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono_tz::Tz;
use protocol::protocol::Event;
use protocol::protocol::MultiServiceSubscriber;
use protocol::protocol::ServiceName;
use std::collections::BTreeMap;
use std::sync::Arc;
use telegram::client::TelegramClient;
use tokio::sync::Mutex;
use weather::forecast::fetcher::{SingleWeatherForecast, WeatherForecast};
use weather::forecast::model::Model;
use weather::station::Station;

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
    station: Station,
    model: Model,
    forecast: Arc<Mutex<BTreeMap<DateTime<Tz>, SingleWeatherForecast>>>,
    telegram_client: Arc<Mutex<TelegramClient>>,
}

impl ForecastNotifier {
    pub async fn new(station: Station, model: Model) -> Self {
        let telegram_client = TelegramClient::start()
            .await
            .expect("Create telegram client");
        Self {
            station,
            model,
            forecast: Arc::new(Mutex::new(BTreeMap::new())),
            telegram_client: Arc::new(Mutex::new(telegram_client)),
        }
    }
}

async fn handle_event(
    model: Model,
    event: WeatherEvents,
    date: &DateTime<Tz>,
    forecast: &Arc<Mutex<BTreeMap<DateTime<Tz>, SingleWeatherForecast>>>,
    telegram_client: &Arc<Mutex<TelegramClient>>,
) -> Result<()> {
    match event {
        WeatherEvents::WeatherForecast(data) => {
            let complete = data.message.complete;
            let new_forecast: BTreeMap<DateTime<Tz>, SingleWeatherForecast> = data
                .message
                .forecast
                .into_iter()
                .filter(|(k, _)| {
                    let dt: DateTime<Tz> = (*k).into();
                    check_dates_match(date, &dt)
                })
                .map(|(k, v)| (k.into(), v))
                .collect();

            if complete {
                let mut forecast = forecast.lock().await;
                forecast.extend(new_forecast);
                if let Some((dt, max_temp)) = forecast.iter().max_by_key(|(_, v)| v.temperature) {
                    let lead_time = max_temp._lead_time;
                    let stdev = model.stdev(lead_time);
                    println!(
                        "Max temperature {:.2}F±{:.2} (68% odds; {}h lead time) at {}",
                        max_temp.temperature.as_fahrenheit(),
                        stdev,
                        lead_time,
                        dt
                    );
                    telegram_client
                        .lock()
                        .await
                        .message()
                        .with_title("📈 Forecast update")
                        .with_item(format!(
                            "Max temp: {:.2}F±{:.2} (68% odds)",
                            max_temp.temperature.as_fahrenheit(),
                            stdev,
                        ))
                        .with_item(format!("Lead time: {}h", lead_time))
                        .with_item(format!("At: {}", dt))
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
        let date = date
            .and_time(NaiveTime::default())
            .and_local_timezone(self.station.timezone())
            .single()
            .context("Expected a sigle timestamp from the station's timezone")?;

        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
        client
            .add_subscription::<WeatherForecast>(ServiceName::WeatherForecast)
            .await?;

        let model = self.model;
        let forecast = self.forecast.clone();
        let telegram_client = self.telegram_client.clone();
        let _event_listener = client
            .listen_all(|event| {
                let forecast = forecast.clone();
                let telegram_client = telegram_client.clone();
                async move {
                    handle_event(model, event, &date, &forecast, &telegram_client).await?;
                    Ok(())
                }
            })
            .await?;

        Ok(())
    }
}
