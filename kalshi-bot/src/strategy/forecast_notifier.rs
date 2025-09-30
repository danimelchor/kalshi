use crate::strategy::strategy::Strategy;
use async_trait::async_trait;
use futures::join;
use protocol::protocol::{Event, MultiServiceSubscriber, ServiceName};
use serde_json::to_string;
use std::sync::{Arc, Mutex};
use tiny_http::{Response, Server};
use weather::forecast::fetcher::WeatherForecast;

#[derive(Debug)]
pub enum WeatherEvents {
    WeatherForecast(Event<WeatherForecast>),
}

impl From<Event<WeatherForecast>> for WeatherEvents {
    fn from(event: Event<WeatherForecast>) -> Self {
        WeatherEvents::WeatherForecast(event)
    }
}

#[derive(Default)]
pub struct ForecastNotifier {
    buffer: Arc<Mutex<Option<WeatherForecast>>>,
}

fn server(buffer: Arc<Mutex<Option<WeatherForecast>>>) {
    // Bind to localhost:8000
    let server = Server::http("0.0.0.0:8000").unwrap();
    println!("Server running on http://0.0.0.0:8000");

    for request in server.incoming_requests() {
        let state = buffer.lock().unwrap();
        let json_response = to_string(&*state).unwrap();
        let response = Response::from_string(json_response);
        request.respond(response).unwrap();
    }
}

#[async_trait]
impl Strategy<WeatherEvents> for ForecastNotifier {
    async fn run(&mut self) -> tokio::io::Result<()> {
        let server_buffer = Arc::clone(&self.buffer);
        let server = tokio::task::spawn_blocking(|| {
            server(server_buffer);
        });

        let mut client = MultiServiceSubscriber::<WeatherEvents>::default();
        client
            .add_subscription::<WeatherForecast>(ServiceName::WeatherForecast)
            .await?;

        let buffer = Arc::clone(&self.buffer);
        let event_listener = client.listen_all(|event| match event {
            WeatherEvents::WeatherForecast(data) => {
                println!("Weather forecast data: {:?}", data);
                *buffer.lock().unwrap() = Some(data.message);
            }
        });

        let (_server_result, _event_listener_result) = join!(server, event_listener);

        Ok(())
    }
}
