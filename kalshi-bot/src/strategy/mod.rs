mod dump_if_temp_higher;
mod forecast_notifier;
pub mod name;
pub mod strategy;
mod utils;

use crate::strategy::{
    dump_if_temp_higher::DumpIfTempHigher, forecast_notifier::ForecastNotifier, name::StrategyName,
    strategy::Strategy,
};
use anyhow::Result;
use chrono::NaiveDate;
use clap::Args;
use weather::{forecast::model::Model, station::Station};

#[derive(Debug, Clone, Args)]
pub struct StrategyCommand {
    name: StrategyName,

    #[arg(short, long)]
    date: NaiveDate,
}

pub async fn run_strategy(command: &StrategyCommand) -> Result<()> {
    match command.name {
        StrategyName::ForecastNotifier => {
            let mut strategy = ForecastNotifier::new(Station::KNYC, Model::HRRR).await;
            strategy.run(&command.date).await.unwrap()
        }
        StrategyName::DumpIfTempHigher => {
            let mut strategy = DumpIfTempHigher::new(Station::KNYC).await;
            strategy.run(&command.date).await.unwrap()
        }
    }

    Ok(())
}
