use crate::proto::name::name_client::NameClient;
pub use crate::proto::name::{
    NameMessage, NameResponse
};
use tonic::transport::Channel;
use tonic::Request;

mod proto;

#[derive(Clone)]
pub struct NameGrpcInterface {
    client: NameClient<Channel>,
}

impl NameGrpcInterface {
    pub async fn new(addr: &'static str, port: u16) -> Self {
        let channel = Channel::from_shared(format!("http://{addr}:{port}"))
            .unwrap()
            .connect()
            .await
            .unwrap();
        Self {
            client: NameClient::new(channel),
        }
    }

    pub async fn name(&mut self, message: NameMessage) -> Option<NameResponse> {
        self.client
            .name(Request::new(message))
            .await
            .map(tonic::Response::into_inner)
            .ok()
    }
}
