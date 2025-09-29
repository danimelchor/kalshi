use bincode::{self, Decode, Encode, encode_to_vec};
use chrono::Utc;
use futures::lock::Mutex;
use futures::stream::{SelectAll, Stream, StreamExt, unfold};
use std::pin::Pin;
use std::sync::Arc;
use std::{marker::PhantomData, path::Path};
use strum_macros;
use tokio::fs;
use tokio::net::UnixListener;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::datetime::SerializableDateTime;

#[derive(strum_macros::Display, Hash, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum ServiceName {
    WeatherForecast,
    GovForecast,
    OpenMeteoForecast,
}

impl ServiceName {
    fn unix_path(&self) -> String {
        let name = self.to_string();
        format!("/tmp/{}.sock", name)
    }
}

#[derive(Debug, Decode, Encode)]
pub struct Event<T> {
    id: u32,
    message: T,
    ts: SerializableDateTime,
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

pub struct ServicePublisher<T> {
    clients: Arc<Mutex<Vec<UnixStream>>>,
    service_name: ServiceName,
    _marker: PhantomData<T>,
}

impl<T> ServicePublisher<T>
where
    T: Encode,
{
    pub async fn new(service_name: ServiceName) -> tokio::io::Result<Self> {
        let path = service_name.unix_path();
        if Path::new(&path).exists() {
            fs::remove_file(&path).await?;
        }

        let listener = UnixListener::bind(&path)?;
        let clients = Arc::new(Mutex::new(Vec::new()));

        let clients_clone = clients.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        clients_clone.lock().await.push(stream);
                    }
                    Err(e) => {
                        eprint!("Failed to accept subscriber connection: {}", e)
                    }
                }
            }
        });

        Ok(Self {
            clients,
            service_name,
            _marker: PhantomData,
        })
    }

    pub async fn publish(&mut self, event: &Event<T>) -> tokio::io::Result<()> {
        let buf = encode_to_vec(event, bincode::config::standard()).unwrap();
        let len = buf.len() as u32;

        let mut clients = self.clients.lock().await;
        let mut failed_clients = Vec::new();

        for (index, client) in clients.iter_mut().enumerate() {
            let mut success = true;

            if client.write_all(&len.to_le_bytes()).await.is_err()
                || client.write_all(&buf).await.is_err()
            {
                success = false;
            }

            if !success {
                failed_clients.push(index);
            }
        }

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
    service: ServiceName,
    _marker: PhantomData<T>,
}

impl<T> ServiceSubscriber<T>
where
    T: Decode<()> + Send + Sync + 'static,
{
    pub async fn new(service: ServiceName) -> tokio::io::Result<Self> {
        let stream = ServiceSubscriber::<T>::subscribe(&service).await?;
        Ok(Self {
            service,
            subscription: stream,
            _marker: PhantomData,
        })
    }

    pub async fn subscribe(service: &ServiceName) -> tokio::io::Result<UnixStream> {
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
                    println!(
                        "Connection to {} failed (attempt {}), retrying in {:?}...",
                        service, retry_count, delay
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    return Err(tokio::io::Error::new(
                        tokio::io::ErrorKind::NotFound,
                        format!(
                            "Failed to connect to {} after {} attempts. Error: {}",
                            service, max_retries, e
                        ),
                    ));
                }
            }
        }
    }

    pub fn listen(
        self,
    ) -> impl Stream<Item = Result<Event<T>, Box<dyn std::error::Error + Send + Sync>>> {
        let service_name_str = self.service.to_string();
        unfold(self.subscription, move |mut stream| {
            let service_name_clone = service_name_str.clone();
            Box::pin(async move {
                // Read message length
                let mut len_buf = [0u8; 4];
                if let Err(e) = stream.read_exact(&mut len_buf).await {
                    eprintln!("Lost connection to {}: {}", service_name_clone, e);
                    return None;
                }

                let len = u32::from_le_bytes(len_buf) as usize;

                // Read message data
                let mut buf = vec![0u8; len];
                if let Err(e) = stream.read_exact(&mut buf).await {
                    eprintln!("Failed to read from {}: {}", service_name_clone, e);
                    return None;
                }

                // Decode the message
                match bincode::decode_from_slice::<Event<T>, _>(&buf, bincode::config::standard()) {
                    Ok((event, _)) => Some((Ok(event), stream)),
                    Err(e) => Some((
                        Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                        stream,
                    )),
                }
            })
        })
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
    pub async fn add_subscription<T>(&mut self, service: ServiceName) -> tokio::io::Result<()>
    where
        T: Decode<()> + Send + Sync + 'static,
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

    pub async fn listen_all<F>(mut self, mut handler: F)
    where
        F: FnMut(E),
    {
        while let Some(event) = self.streams.next().await {
            handler(event);
        }
    }
}
