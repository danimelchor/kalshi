use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum StrategyName {
    ForecastNotifier,
    DumpIfTempHigher,
}
