use super::request_log::ServerLogger;
use crate::{
    datastore::DatastoreWrapper,
    proto::wallguard::{
        wall_guard_server::WallGuard, Authentication, CommonResponse, ConfigSnapshot,
        HeartbeatRequest, HeartbeatResponse, LoginRequest, Packets, SetupRequest, StatusRequest,
        StatusResponse,
    },
};
use std::net::IpAddr;
use std::sync::mpsc::Sender;

use crate::proto::wallguard::Logs;
use tonic::{Request, Response, Status};

pub(crate) struct WallGuardImpl {
    pub(crate) datastore: DatastoreWrapper,
    pub(crate) ip_info_tx: Sender<Option<IpAddr>>,
}

#[tonic::async_trait]
impl WallGuard for WallGuardImpl {
    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let addr = ServerLogger::extract_address(&request);
        let received_at = chrono::Utc::now();
        let result = self.heartbeat_impl(request).await;
        ServerLogger::log_response(&result, &addr, "/heartbeat", received_at);
        result.map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn setup(
        &self,
        request: Request<SetupRequest>,
    ) -> Result<Response<CommonResponse>, Status> {
        let addr = ServerLogger::extract_address(&request);
        let received_at = chrono::Utc::now();
        let result = self.setup_impl(request).await;
        ServerLogger::log_response(&result, &addr, "/setup", received_at);
        result.map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<Authentication>, Status> {
        let addr = ServerLogger::extract_address(&request);
        let received_at = chrono::Utc::now();
        let result = self.login_impl(request).await;
        ServerLogger::log_response(&result, &addr, "/login", received_at);
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
        let addr = ServerLogger::extract_address(&request);
        let received_at = chrono::Utc::now();
        let result = self.handle_logs_impl(request).await;
        ServerLogger::log_response(&result, &addr, "/handle_logs", received_at);
        result.map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let addr = ServerLogger::extract_address(&request);
        let received_at = chrono::Utc::now();
        let result = self.device_status_impl(request).await;
        ServerLogger::log_response(&result, &addr, "/status", received_at);
        result.map_err(|e| Status::internal(format!("{e:?}")))
    }
}
