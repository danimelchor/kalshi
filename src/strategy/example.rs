use crate::{
    datasource::{
        datasource::DataSource,
        example::{ExampleComData, ExampleComDataSource},
    },
    protocol::protocol::{Event, ServiceName},
    strategy::strategy::Strategy,
};

enum ExampleEvents {
    ExampleComData(Event<ExampleComData>),
}

pub struct ExampleStrategy();

impl Strategy<ExampleEvents> for ExampleStrategy {
    fn name() -> String {
        "example".into()
    }
    fn datasources() -> Vec<ServiceName> {
        vec![ExampleComDataSource::service_name()]
    }

    async fn run(&mut self) {}
}
