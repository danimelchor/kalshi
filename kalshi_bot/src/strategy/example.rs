use async_trait::async_trait;

use crate::{
    datasource::{
        datasource::DataSource,
        example::{ExampleComData, ExampleComDataSource},
    },
    protocol::protocol::{Event, MultiServiceSubscriber, ServiceName},
    strategy::strategy::Strategy,
};

#[derive(Debug)]
pub enum ExampleEvents {
    ExampleComData(Event<ExampleComData>),
}

impl From<Event<ExampleComData>> for ExampleEvents {
    fn from(event: Event<ExampleComData>) -> Self {
        ExampleEvents::ExampleComData(event)
    }
}

#[derive(Default)]
pub struct ExampleStrategy();

#[async_trait]
impl Strategy<ExampleEvents> for ExampleStrategy {
    fn name() -> String {
        "example".into()
    }

    fn datasources() -> Vec<ServiceName> {
        vec![ExampleComDataSource::service_name()]
    }

    async fn run(&mut self) -> tokio::io::Result<()> {
        let mut client = MultiServiceSubscriber::<ExampleEvents>::default();
        client
            .add_subscription::<ExampleComData>(ServiceName::GovForecast)
            .await?;

        client
            .listen_all(|event| match event {
                ExampleEvents::ExampleComData(data) => println!("Example.com data: {:?}", data),
            })
            .await;

        Ok(())
    }
}
