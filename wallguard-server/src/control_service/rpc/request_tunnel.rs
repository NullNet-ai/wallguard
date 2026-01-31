use crate::control_service::service::WallGuardService;
use crate::reverse_tunnel::TunnelInstance;
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use wallguard_common::protobuf::wallguard_tunnel::ClientFrame;
use wallguard_common::protobuf::wallguard_tunnel::reverse_tunnel_server::ReverseTunnel;

impl WallGuardService {
    pub(crate) async fn request_tunnel_impl(
        &self,
        request: Request<Streaming<ClientFrame>>,
    ) -> Result<Response<<WallGuardService as ReverseTunnel>::RequestTunnelStream>, Status> {
        let request = request.into_inner();

        let (sender, receiver) = mpsc::channel(64);

        let tunnel_instance = TunnelInstance::new(request, sender);
        self.context.tunnel.on_new_tunnel_opened(tunnel_instance);

        Ok(Response::new(ReceiverStream::new(receiver)))
    }
}

// TODO: Remove
