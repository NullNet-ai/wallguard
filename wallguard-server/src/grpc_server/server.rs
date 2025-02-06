use crate::{
    datastore::DatastoreWrapper,
    proto::wallguard::{
        wall_guard_server::WallGuard, Authentication, CommonResponse, ConfigSnapshot, Empty,
        HeartbeatRequest, LoginRequest, Packets, SetupRequest,
    },
};
use tonic::{Request, Response, Status};

pub(crate) struct WallGuardImpl {
    pub(crate) tx: async_channel::Sender<Packets>,
    pub(crate) datastore: Option<DatastoreWrapper>,
}

#[tonic::async_trait]
impl WallGuard for WallGuardImpl {
    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<CommonResponse>, Status> {
        WallGuardImpl::log_request(&request, "heartbeat");
        self.heartbeat_impl(request).await
    }

    async fn setup(
        &self,
        request: Request<SetupRequest>,
    ) -> Result<Response<CommonResponse>, Status> {
        WallGuardImpl::log_request(&request, "setup");
        self.setup_impl(request).await
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<Authentication>, Status> {
        WallGuardImpl::log_request(&request, "login");
        self.login_impl(request).await
    }

    async fn handle_packets(&self, request: Request<Packets>) -> Result<Response<Empty>, Status> {
        WallGuardImpl::log_request(&request, "handle_packets");
        self.handle_packets_impl(request).await
    }

    async fn handle_config(
        &self,
        request: Request<ConfigSnapshot>,
    ) -> Result<Response<CommonResponse>, Status> {
        WallGuardImpl::log_request(&request, "handle_config");
        self.handle_config_impl(request).await
    }
}
