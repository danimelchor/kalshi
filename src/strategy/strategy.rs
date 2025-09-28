use crate::protocol::protocol::ServiceName;

pub trait Strategy<T> {
    fn name() -> String;
    fn datasources() -> Vec<ServiceName>;
    async fn run(&mut self);
}
