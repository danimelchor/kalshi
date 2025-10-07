use crate::strategy::strategy::Strategy;
use crate::strategy::utils::check_dates_match;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono_tz::Tz;
use protocol::protocol::{Event, MultiServiceSubscriber, ServiceName};
use std::collections::BTreeMap;
use std::sync::Arc;
use telegram::client::TelegramClient;
use tokio::sync::Mutex;
use weather::forecast::fetcher::{SingleWeatherForecast, WeatherForecast};
use weather::forecast::model::Model;
use weather::temperature::Temperature;
use weather::{
    observations::{
        nws_daily_report::NWSDailyReport, nws_hourly_table::NWSHourlyTableTemperatures,
        nws_hourly_timeseries::NWSHourlyTimeseriesTemperatures,
    },
    station::Station,
};

#[derive(Debug)]
pub enum WeatherEvents {
    WeatherForecast(Event<WeatherForecast>),
    HourlyWeatherTimeseries(NWSHourlyTimeseriesTemperatures),
    HourlyWeatherTable(NWSHourlyTableTemperatures),
    DailyWeatherReport(NWSDailyReport),
}

impl From<Event<WeatherForecast>> for WeatherEvents {
    fn from(event: Event<WeatherForecast>) -> Self {
        WeatherEvents::WeatherForecast(event)
    }
}

impl From<Event<NWSHourlyTimeseriesTemperatures>> for WeatherEvents {
    fn from(event: Event<NWSHourlyTimeseriesTemperatures>) -> Self {
        WeatherEvents::HourlyWeatherTimeseries(event.message)
    }
}

impl From<Event<NWSHourlyTableTemperatures>> for WeatherEvents {
    fn from(event: Event<NWSHourlyTableTemperatures>) -> Self {
        WeatherEvents::HourlyWeatherTable(event.message)
    }
}

impl From<Event<NWSDailyReport>> for WeatherEvents {
    fn from(event: Event<NWSDailyReport>) -> Self {
        WeatherEvents::DailyWeatherReport(event.message)
    }
}

pub struct WeatherBetter {
    station: Station,
    model: Model,
    observed_max: Arc<Mutex<Option<Temperature>>>,
    forecast: Arc<Mutex<BTreeMap<DateTime<Tz>, SingleWeatherForecast>>>,
    telegram_client: Arc<Mutex<TelegramClient>>,
}

impl WeatherBetter {
    pub async fn new(station: Station, model: Model) -> Self {
        let telegram_client = TelegramClient::start()
            .await
            .expect("Create telegram client");
        Self {
            station,
            model,
            observed_max: Arc::new(Mutex::new(None)),
            forecast: Arc::new(Mutex::new(BTreeMap::new())),
            telegram_client: Arc::new(Mutex::new(telegram_client)),
        }
    }
}

async fn maybe_update_max_temp(
    seen: Temperature,
    observed_max: &Arc<Mutex<Option<Temperature>>>,
    telegram_client: &Arc<Mutex<TelegramClient>>,
) -> Result<()> {
    let mut guard = observed_max.lock().await;
    let send_telegram = async |temp: Temperature| {
        println!("Max observation: {}F ", temp.as_fahrenheit(),);
        telegram_client
            .lock()
            .await
            .message()
            .with_title("☀️ Max observation")
            .with_item(format!("Max observation: {}F", temp.as_fahrenheit()))
            .send()
            .await
    };

    match guard.as_ref() {
        None => {
            *guard = Some((seen));
            send_telegram(seen).await?;
        }
        Some(max_t) => {
            if seen > *max_t {
                *guard = Some((seen));
                send_telegram(seen).await?;
            }
        }
    }

    Ok(())
}

async fn handle_event(
    model: Model,
    event: WeatherEvents,
    date: &DateTime<Tz>,
    forecast: &Arc<Mutex<BTreeMap<DateTime<Tz>, SingleWeatherForecast>>>,
    observed_max: &Arc<Mutex<Option<Temperature>>>,
    telegram_client: &Arc<Mutex<TelegramClient>>,
) -> Result<()> {
    let maybe_update =
        async |seen| maybe_update_max_temp(seen, observed_max, telegram_client).await;

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
        }
        WeatherEvents::HourlyWeatherTimeseries(data) => {
            let data: Vec<_> = data
                .0
                .into_iter()
                .filter(|obs| {
                    let dt: DateTime<Tz> = obs.datetime.into();
                    check_dates_match(date, &dt)
                })
                .collect();

            if let Some(max_temp) = data.iter().max_by_key(|t| &t.temperature) {
                maybe_update(max_temp.temperature).await?;
            }

            if let Some(max_six_h_temp) = data
                .iter()
                .filter_map(|t| t.six_hr_max_temperature.as_ref())
                .max()
            {
                maybe_update(*max_six_h_temp).await?;
            }
        }
        WeatherEvents::HourlyWeatherTable(data) => {
            let data: Vec<_> = data
                .0
                .into_iter()
                .filter(|obs| {
                    let dt: DateTime<Tz> = obs.datetime.into();
                    check_dates_match(date, &dt)
                })
                .collect();

            if let Some(max_temp) = data.iter().max_by_key(|t| &t.temperature) {
                maybe_update(max_temp.temperature).await?;
            }

            if let Some(max_six_h_temp) = data
                .iter()
                .filter_map(|t| t.six_hr_max_temperature.as_ref())
                .max()
            {
                maybe_update(*max_six_h_temp).await?;
            }
        }
        WeatherEvents::DailyWeatherReport(data) => {
            let dt: DateTime<Tz> = data.datetime.into();
            if check_dates_match(date, &dt) {
                maybe_update(data.max_temperature).await?;
            }
        }
    }
    Ok(())
}

#[async_trait]
impl Strategy<WeatherEvents> for WeatherBetter {
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
        client
            .add_subscription::<NWSHourlyTimeseriesTemperatures>(
                ServiceName::HourlyWeatherTimeseries,
            )
            .await?;
        client
            .add_subscription::<NWSHourlyTableTemperatures>(ServiceName::HourlyWeatherTable)
            .await?;
        client
            .add_subscription::<NWSDailyReport>(ServiceName::DailyWeatherReport)
            .await?;

        let model = self.model;
        let forecast = self.forecast.clone();
        let observed_max = self.observed_max.clone();
        let telegram_client = self.telegram_client.clone();
        let _event_listener = client
            .listen_all(|event| {
                let forecast = forecast.clone();
                let observed_max = observed_max.clone();
                let telegram_client = telegram_client.clone();
                async move {
                    handle_event(
                        model,
                        event,
                        &date,
                        &forecast,
                        &observed_max,
                        &telegram_client,
                    )
                    .await?;
                    Ok(())
                }
            })
            .await?;

        Ok(())
    }
}
