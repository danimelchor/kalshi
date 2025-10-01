pub mod dump_if_temp_higher;
pub mod forecast_notifier;
pub mod name;
pub mod strategy;

use crate::strategy::{
    dump_if_temp_higher::DumpIfTempHigher, forecast_notifier::ForecastNotifier, name::StrategyName,
    strategy::Strategy,
};
use anyhow::Result;
use chrono::NaiveDate;
use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct StrategyCommand {
    name: StrategyName,

    #[arg(short, long)]
    date: NaiveDate,
}

pub async fn run_strategy(command: &StrategyCommand) -> Result<()> {
    match command.name {
        StrategyName::ForecastNotifier => {
            let mut strategy = ForecastNotifier::default();
            strategy.run(&command.date).await.unwrap()
        }
        StrategyName::DumpIfTempHigher => {
            let mut strategy = DumpIfTempHigher::default();
            strategy.run(&command.date).await.unwrap()
        }
    }

    Ok(())
}
