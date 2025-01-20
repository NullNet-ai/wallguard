use crate::proto::wallguard::wall_guard_client::WallGuardClient;
pub use crate::proto::wallguard::{SampleMessage, SampleResponse};
use tonic::transport::Channel;
use tonic::Request;

mod proto;

#[derive(Clone)]
pub struct WallGuardGrpcInterface {
    client: WallGuardClient<Channel>,
}

impl WallGuardGrpcInterface {
    pub async fn new(addr: &'static str, port: u16) -> Self {
        let channel = Channel::from_shared(format!("http://{addr}:{port}"))
            .unwrap()
            .connect()
            .await
            .unwrap();
        Self {
            client: WallGuardClient::new(channel),
        }
    }

    pub async fn sample(&mut self, message: SampleMessage) -> Option<SampleResponse> {
        self.client
            .sample(Request::new(message))
            .await
            .map(tonic::Response::into_inner)
            .ok()
    }
}
