use kalshi::datasource::{datasource::DataSource, example::{ExampleComDataSource}};

#[tokio::main]
async fn main()  {
    let mut source = ExampleComDataSource::new();
    source.run().await.unwrap()
}
