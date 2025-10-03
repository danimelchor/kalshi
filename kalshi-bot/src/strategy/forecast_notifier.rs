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
    last_max: Arc<Mutex<Option<SingleWeatherForecast>>>,
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
            last_max: Arc::new(Mutex::new(None)),
            forecast: Arc::new(Mutex::new(BTreeMap::new())),
            telegram_client: Arc::new(Mutex::new(telegram_client)),
        }
    }
}

async fn handle_event(
    model: Model,
    event: WeatherEvents,
    date: &DateTime<Tz>,
    last_max: &Arc<Mutex<Option<SingleWeatherForecast>>>,
    forecast: &Arc<Mutex<BTreeMap<DateTime<Tz>, SingleWeatherForecast>>>,
    telegram_client: &Arc<Mutex<TelegramClient>>,
) -> Result<()> {
    match event {
        WeatherEvents::WeatherForecast(data) => {
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

            let mut forecast = forecast.lock().await;
            let mut last_max = last_max.lock().await;

            forecast.extend(new_forecast);
            if let Some((dt, max_temp)) = forecast.iter().max_by_key(|(_, v)| v.temperature) {
                // Don't spam if we've already told the user about this max
                if last_max.is_some() && last_max.unwrap().temperature == max_temp.temperature {
                    return Ok(());
                }

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

                *last_max = Some(*max_temp);
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
        let last_max = self.last_max.clone();
        let telegram_client = self.telegram_client.clone();
        let _event_listener = client
            .listen_all(|event| {
                let forecast = forecast.clone();
                let last_max = last_max.clone();
                let telegram_client = telegram_client.clone();
                async move {
                    handle_event(model, event, &date, &last_max, &forecast, &telegram_client)
                        .await?;
                    Ok(())
                }
            })
            .await?;

        Ok(())
    }
}
