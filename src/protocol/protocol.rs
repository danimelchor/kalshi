use bincode::{self, Decode, Encode, encode_to_vec};
use futures::stream::{Stream, unfold};
use std::{marker::PhantomData, path::Path};
use strum_macros;
use tokio::fs;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

#[derive(strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum ServiceName {
    GovForecast,
    OpenMeteoForecast,
}

impl ServiceName {
    fn unix_path(&self) -> String {
        let name = self.to_string();
        format!("/tmp/{}.sock", name)
    }
}

#[derive(Decode, Encode)]
pub struct Event<T> {
    id: u32,
    message: T,
}

impl<T> Event<T> {
    pub fn new(id: u32, message: T) -> Self {
        Self { id, message }
    }
}

pub struct ServicePublisher<T> {
    stream: UnixStream,
    _marker: PhantomData<T>,
}

impl<T> ServicePublisher<T>
where
    T: Encode,
{
    pub async fn new(name: ServiceName) -> tokio::io::Result<Self> {
        let path = name.unix_path();
        let stream = UnixStream::connect(path).await?;
        Ok(Self {
            stream,
            _marker: PhantomData,
        })
    }

    pub async fn publish(&mut self, event: &Event<T>) -> tokio::io::Result<()> {
        let buf = encode_to_vec(event, bincode::config::standard()).unwrap();
        let len = buf.len() as u32;
        self.stream.write_all(&len.to_le_bytes()).await?;
        self.stream.write_all(&buf).await?;
        Ok(())
    }
}

pub struct ServiceSubscriber<T> {
    stream: UnixStream,
    _marker: PhantomData<T>,
}

impl<T> ServiceSubscriber<T>
where
    T: Decode<()> + Send + 'static,
{
    pub async fn new(name: ServiceName) -> tokio::io::Result<Self> {
        let path = name.unix_path();
        if Path::new(&path).exists() {
            fs::remove_file(&path).await?;
        }
        let stream = UnixStream::connect(path).await?;
        Ok(Self {
            stream,
            _marker: PhantomData,
        })
    }

    pub fn subscribe(
        self,
    ) -> impl Stream<Item = Result<T, Box<dyn std::error::Error + Send + Sync>>> {
        unfold(self.stream, |mut stream| async move {
            // Read message length
            let mut len_buf = [0u8; 4];
            if let Err(e) = stream.read_exact(&mut len_buf).await {
                eprintln!("Failed to read length: {}", e);
                return None;
            }

            let len = u32::from_le_bytes(len_buf) as usize;

            // Read message data
            let mut buf = vec![0u8; len];
            if let Err(e) = stream.read_exact(&mut buf).await {
                eprintln!("Failed to read data: {}", e);
                return None;
            }

            // Decode the message
            match bincode::decode_from_slice::<T, _>(&buf, bincode::config::standard()) {
                Ok((event, _)) => Some((Ok(event), stream)),
                Err(e) => Some((
                    Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                    stream,
                )),
            }
        })
    }
}
