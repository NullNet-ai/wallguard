use tonic::{Request, Response, Status};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{Empty, Packets},
};

impl WallGuardImpl {
    pub(crate) async fn handle_packets_impl(
        &self,
        request: Request<Packets>,
    ) -> Result<Response<Empty>, Status> {
        self.tx
            .try_send(request.into_inner())
            .map_err(|_| Status::internal("Failed to send packets to workers"))?;

        Ok(Response::new(Empty {}))
    }
}
