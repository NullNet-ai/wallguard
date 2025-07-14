use crate::control_service::service::WallGuardService;
use wallguard_common::protobuf::wallguard_commands::ClientMessage;
use wallguard_common::protobuf::wallguard_service::wall_guard_server::WallGuard;

use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

impl WallGuardService {
    pub(crate) async fn control_channel_impl(
        &self,
        request: Request<Streaming<ClientMessage>>,
    ) -> Result<Response<<WallGuardService as WallGuard>::ControlChannelStream>, Status> {
        let (sender, receiver) = mpsc::channel(64);

        self.context.orchestractor.on_new_connection(
            request.into_inner(),
            sender,
            self.context.clone(),
        );

        Ok(Response::new(ReceiverStream::new(receiver)))
    }
}
