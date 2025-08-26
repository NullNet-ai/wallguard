use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;
use tokio::sync::mpsc;
use tonic::Request;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Streaming, transport::Channel};

use crate::protobuf::wallguard_tunnel::{
    ClientFrame, ServerFrame, reverse_tunnel_client::ReverseTunnelClient,
};

#[derive(Clone, Debug)]
pub struct WallGuardTunnelGrpcInterface {
    client: ReverseTunnelClient<Channel>,
}

impl WallGuardTunnelGrpcInterface {
    #[allow(clippy::missing_panics_doc)]
    pub async fn new(addr: &str, port: u16) -> Result<Self, Error> {
        let addr = format!("http://{addr}:{port}");

        let channel = Channel::from_shared(addr)
            .expect("Failed to parse address")
            .timeout(Duration::from_secs(10))
            .keep_alive_timeout(Duration::from_secs(10))
            .connect()
            .await
            .handle_err(location!())?;

        let client = ReverseTunnelClient::new(channel).max_decoding_message_size(50 * 1024 * 1024);

        Ok(Self { client })
    }

    pub async fn request_control_channel(
        &self,
        receiver: mpsc::Receiver<ClientFrame>,
    ) -> Result<Streaming<ServerFrame>, Error> {
        let receiver = ReceiverStream::new(receiver);

        let response = self
            .client
            .clone()
            .request_tunnel(Request::new(receiver))
            .await
            .handle_err(location!())?;

        Ok(response.into_inner())
    }
}
