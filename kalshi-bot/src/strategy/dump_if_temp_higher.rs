use std::sync::Arc;

use crate::strategy::{strategy::Strategy, utils::check_dates_match};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime};
use chrono_tz::Tz;
use protocol::protocol::{Event, MultiServiceSubscriber, ServiceName};
use tokio::sync::Mutex;
use weather::{
    observations::{
        nws_daily_report::NWSDailyReport, nws_hourly_table::NWSHourlyTableTemperatures,
        nws_hourly_timeseries::NWSHourlyTimeseriesTemperatures,
    },
    station::Station,
    temperature::Temperature,
};

#[derive(Debug)]
pub enum WeatherEvents {
    HourlyWeatherTimeseries(NWSHourlyTimeseriesTemperatures),
    HourlyWeatherTable(NWSHourlyTableTemperatures),
    DailyWeatherReport(NWSDailyReport),
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

async fn maybe_update_max_temp(seen: Temperature, observed_max: &Arc<Mutex<Option<Temperature>>>) {
    let mut guard = observed_max.lock().await;
    match guard.as_ref() {
        None => *guard = Some(seen),
        Some(max_t) => {
            if seen > *max_t {
                *guard = Some(seen)
            }
        }
    }
}

async fn handle_event(
    event: WeatherEvents,
    date: &DateTime<Tz>,
    observed_max: Arc<Mutex<Option<Temperature>>>,
) -> Result<()> {
    match event {
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
                maybe_update_max_temp(max_temp.temperature, &observed_max).await;
            }

            if let Some(max_six_h_temp) = data
                .iter()
                .filter_map(|t| t.six_hr_max_temperature.as_ref())
                .max()
            {
                maybe_update_max_temp(*max_six_h_temp, &observed_max).await;
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
                maybe_update_max_temp(max_temp.temperature, &observed_max).await;
            }

            if let Some(max_six_h_temp) = data
                .iter()
                .filter_map(|t| t.six_hr_max_temperature.as_ref())
                .max()
            {
                maybe_update_max_temp(*max_six_h_temp, &observed_max).await;
            }
        }
        WeatherEvents::DailyWeatherReport(data) => {
            println!("Daily weather report: {:?}", data);
            let dt: DateTime<Tz> = data.datetime.into();
            if check_dates_match(date, &dt) {
                maybe_update_max_temp(data.max_temperature, &observed_max).await;
            }
        }
    }
    Ok(())
}

pub struct DumpIfTempHigher {
    station: Station,
    observed_max: Arc<Mutex<Option<Temperature>>>,
}

impl DumpIfTempHigher {
    pub fn new(station: Station) -> Self {
        Self {
            station,
            observed_max: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Strategy<WeatherEvents> for DumpIfTempHigher {
    async fn run(&mut self, date: &NaiveDate) -> Result<()> {
        let date = date
            .and_time(NaiveTime::default())
            .and_local_timezone(self.station.timezone())
            .single()
            .context("Expected a sigle timestamp from the station's timezone")?;

        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
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

        let observed_max = self.observed_max.clone();
        client
            .listen_all(|event| {
                let observed_max = observed_max.clone();
                async move { handle_event(event, &date, observed_max).await }
            })
            .await?;

        Ok(())
    }
}
