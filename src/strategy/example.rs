pub struct ExampleSubscriber {}

impl ServiceSubscriber<T> for ExampleSubscriber
where
    T: Decode<()> + Send + Sync + 'static,
{
    pub async fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            _marker: PhantomData,
        }
    }

    pub async fn subscribe(&mut self, service: ServiceName) -> tokio::io::Result<()> {
        let path = service.unix_path();

        // Retry connection with exponential backoff
        let mut retry_count = 0;
        let max_retries = 5;

        loop {
            match UnixStream::connect(&path).await {
                Ok(stream) => {
                    println!("Subscribed to {}", service);
                    self.subscriptions.insert(service, stream);
                    return Ok(());
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
        let streams = self
            .subscriptions
            .into_iter()
            .map(|(service_name, stream)| {
                let service_name_str = service_name.to_string();
                unfold(stream, move |mut stream| {
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
                        match bincode::decode_from_slice::<Event<T>, _>(
                            &buf,
                            bincode::config::standard(),
                        ) {
                            Ok((event, _)) => Some((Ok(event), stream)),
                            Err(e) => Some((
                                Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                                stream,
                            )),
                        }
                    })
                })
            })
            .collect::<Vec<_>>();

        // Merge all streams into one
        futures::stream::select_all(streams)
    }
}
