use super::request_log::ServerLogger;
use crate::app_context::AppContext;
use crate::proto::wallguard::{
    CommonResponse, ConfigSnapshot, HeartbeatRequest, HeartbeatResponse, Logs, Packets,
    wall_guard_server::WallGuard,
};
use crate::proto::wallguard::{ControlChannelRequest, ControlChannelResponse};
use std::net::IpAddr;
use std::sync::mpsc::Sender;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

pub(crate) struct WallGuardImpl {
    pub(crate) context: AppContext,
    pub(crate) ip_info_tx: Sender<Option<IpAddr>>,
}

#[tonic::async_trait]
impl WallGuard for WallGuardImpl {
    type HeartbeatStream = ReceiverStream<Result<HeartbeatResponse, Status>>;

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<Self::HeartbeatStream>, Status> {
        let result = self.heartbeat_impl(request).await;
        result.map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn handle_packets(
        &self,
        request: Request<Packets>,
    ) -> Result<Response<CommonResponse>, Status> {
        let addr = ServerLogger::extract_address(&request);
        let received_at = chrono::Utc::now();
        let result = self.handle_packets_impl(request).await;
        ServerLogger::log_response(&result, &addr, "/handle_packets", received_at);
        result.map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn handle_config(
        &self,
        request: Request<ConfigSnapshot>,
    ) -> Result<Response<CommonResponse>, Status> {
        let addr = ServerLogger::extract_address(&request);
        let received_at = chrono::Utc::now();
        let result = self.handle_config_impl(request).await;
        ServerLogger::log_response(&result, &addr, "/handle_config", received_at);
        result.map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn handle_logs(
        &self,
        request: Request<Logs>,
    ) -> Result<Response<CommonResponse>, Status> {
        // do not log inside here, otherwise it will loop
        let result = self.handle_logs_impl(request).await;
        result.map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn request_control_channel(
        &self,
        request: Request<ControlChannelRequest>,
    ) -> Result<Response<ControlChannelResponse>, Status> {
        let addr = ServerLogger::extract_address(&request);
        let received_at = chrono::Utc::now();
        let result = self.request_control_channel_impl(request).await;
        ServerLogger::log_response(&result, &addr, "/heartbeat", received_at);
        result.map_err(|e| Status::internal(format!("{e:?}")))
    }
}
