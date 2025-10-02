use anyhow::{Context, Result, anyhow};
use async_stream::stream;
use chrono::Utc;
use futures::lock::Mutex;
use futures::stream::{SelectAll, Stream, StreamExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use std::{marker::PhantomData, path::Path};
use strum_macros;
use tokio::fs;
use tokio::net::UnixListener;
use tokio::sync::RwLock;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::datetime::DateTimeZoned;

#[derive(strum_macros::Display, Hash, PartialEq, Eq, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum ServiceName {
    Telegram,
    WeatherForecast,
    HourlyWeatherTimeseries,
    HourlyWeatherTable,
    DailyWeatherReport,
}

impl ServiceName {
    fn unix_path(&self) -> String {
        let name = self.to_string();
        format!("/tmp/{}.sock", name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event<T> {
    id: u32,
    pub message: T,
    pub ts: DateTimeZoned,
}

impl<T> Event<T> {
    pub fn new(id: u32, message: T) -> Self {
        Self {
            id,
            message,
            ts: Utc::now().into(),
        }
    }
}

pub async fn create_unix_bind(service_name: ServiceName) -> Result<UnixListener> {
    let path = service_name.unix_path();
    if Path::new(&path).exists() {
        fs::remove_file(&path).await?;
    }
    UnixListener::bind(&path).context("Creating unix listener")
}

pub async fn create_unix_stream(service: ServiceName) -> Result<UnixStream> {
    let path = service.unix_path();

    // Retry connection with exponential backoff
    let mut retry_count = 0;
    let max_retries = 5;

    loop {
        match UnixStream::connect(&path).await {
            Ok(stream) => {
                println!("Subscribed to {}", service);
                return Ok(stream);
            }
            Err(_) if retry_count < max_retries => {
                retry_count += 1;
                let delay = std::time::Duration::from_millis(100 * (1 << retry_count));
                tokio::time::sleep(delay).await;
            }
            Err(e) => {
                return Err(anyhow!(
                    "Failed to connect to {} after {} attempts. Error: {}",
                    service,
                    max_retries,
                    e
                ));
            }
        }
    }
}

pub async fn write_one<T: Serialize>(message: &Event<T>, stream: &mut UnixStream) -> Result<()> {
    let buf = bitcode::serialize(&message).context("Serializing telegram message")?;
    let len = buf.len() as u32;
    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(&buf).await?;
    Ok(())
}

pub async fn read<T: DeserializeOwned>(stream: &mut UnixStream) -> Result<Event<T>> {
    // Read message length
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .context("Lost connection")?;

    let len = u32::from_le_bytes(len_buf) as usize;

    // Read message data
    let mut buf = vec![0u8; len];
    stream
        .read_exact(&mut buf)
        .await
        .context("Failed to read full message")?;

    // Deserialize the message
    bitcode::deserialize::<Event<T>>(&buf).context("Deserializing message into type")
}

pub struct ServicePublisher<T> {
    clients: Arc<Mutex<Vec<UnixStream>>>,
    service_name: ServiceName,
    buffer: Arc<RwLock<Vec<Event<T>>>>,
}

impl<T> ServicePublisher<T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub async fn new(service_name: ServiceName) -> Result<Self> {
        let listener = create_unix_bind(service_name).await?;
        let clients = Arc::new(Mutex::new(Vec::new()));
        let buffer = Arc::new(RwLock::new(Vec::new()));

        let clients_clone = clients.clone();
        let buffer_clone = buffer.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Err(e) => {
                        eprint!("Failed to accept subscriber connection: {}", e)
                    }
                    Ok((mut client, _addr)) => {
                        let mut ok = true;
                        for event in buffer_clone.read().await.iter() {
                            if write_one(event, &mut client).await.is_err() {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            clients_clone.lock().await.push(client);
                        }
                    }
                }
            }
        });

        Ok(Self {
            clients,
            service_name,
            buffer,
        })
    }

    pub async fn publish(&mut self, event: Event<T>) -> Result<()> {
        let mut clients = self.clients.lock().await;

        let mut failed_clients = Vec::new();
        for (index, client) in clients.iter_mut().enumerate() {
            if write_one(&event, client).await.is_err() {
                failed_clients.push(index);
            }
        }

        // Store the sent event
        self.buffer.write().await.push(event);

        // Remove failed clients (in reverse order to maintain indices)
        for &index in failed_clients.iter().rev() {
            clients.remove(index);
            println!("Removed disconnected subscriber from {}", self.service_name);
        }

        Ok(())
    }
}

pub struct ServiceSubscriber<T> {
    subscription: UnixStream,
    _marker: PhantomData<T>,
}

impl<T> ServiceSubscriber<T>
where
    T: for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    pub async fn new(service: ServiceName) -> Result<Self> {
        let stream = create_unix_stream(service).await?;
        Ok(Self {
            subscription: stream,
            _marker: PhantomData,
        })
    }

    pub fn listen(self) -> impl Stream<Item = Result<Event<T>>> + Send {
        let mut subscription = self.subscription;
        stream! {
            loop {
                let message = read(&mut subscription).await;
                yield message;
            }
        }
    }
}

pub struct MultiServiceSubscriber<E> {
    streams: SelectAll<Pin<Box<dyn Stream<Item = E> + Send>>>,
}

impl<E> Default for MultiServiceSubscriber<E> {
    fn default() -> Self {
        Self {
            streams: SelectAll::new(),
        }
    }
}

impl<E> MultiServiceSubscriber<E> {
    pub async fn add_subscription<T>(&mut self, service: ServiceName) -> Result<()>
    where
        T: for<'de> Deserialize<'de> + Send + Sync + 'static,
        Event<T>: Into<E>,
    {
        let subscriber = ServiceSubscriber::<T>::new(service).await?;
        let stream = Box::pin(subscriber.listen().filter_map(|result| async move {
            match result {
                Ok(event) => Some(event.into()),
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    None
                }
            }
        }));
        self.streams.push(stream);
        Ok(())
    }

    pub async fn listen_all<F, Fut>(mut self, mut handler: F) -> Result<()>
    where
        F: FnMut(E) -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        while let Some(event) = self.streams.next().await {
            handler(event).await?;
        }
        Ok(())
    }
}
