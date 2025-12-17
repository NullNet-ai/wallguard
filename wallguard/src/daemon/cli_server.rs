use wallguard_common::protobuf::wallguard_cli::Caps;
use wallguard_common::protobuf::wallguard_cli::CommonResponse;
use wallguard_common::protobuf::wallguard_cli::JoinOrgReq;
use wallguard_common::protobuf::wallguard_cli::Status;
use wallguard_common::protobuf::wallguard_cli::wallguard_cli_server::WallguardCli;
use crate::daemon::Daemon;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct CliServer {
    inner: Arc<Mutex<Daemon>>,
}

impl From<Arc<Mutex<Daemon>>> for CliServer {
    fn from(inner: Arc<Mutex<Daemon>>) -> Self {
        Self { inner }
    }
}

#[tonic::async_trait]
impl WallguardCli for CliServer {
    async fn get_status(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<Status>, tonic::Status> {
        let status = self.inner.lock().await.get_status();
        Ok(tonic::Response::from(status))
    }

    async fn get_capabilities(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<Caps>, tonic::Status> {
        use crate::daemon::state::DaemonState;

        let capabilities = match &self.inner.lock().await.state {
            DaemonState::Connected(control_channel) => {
                let ctx = control_channel.get_context();
                let manager = ctx.transmission_manager.lock().await;

                Caps {
                    traffic: manager.has_packet_capture(),
                    telemetry: manager.has_resource_monitoring(),
                    sysconfig: manager.has_sysconf_monitoring(),
                }
            }
            _ => Caps::default(),
        };

        let response = tonic::Response::from(capabilities);
        Ok(response)
    }

    async fn join_org(
        &self,
        request: tonic::Request<JoinOrgReq>,
    ) -> Result<tonic::Response<CommonResponse>, tonic::Status> {
        let installation_code = request.into_inner().installation_code;

        let response = match Daemon::join_org(self.inner.clone(), installation_code).await {
            Ok(_) => CommonResponse {
                success: true,
                message: String::from("OK"),
            },
            Err(message) => CommonResponse {
                success: false,
                message,
            },
        };

        Ok(tonic::Response::from(response))
    }

    async fn leave_org(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<CommonResponse>, tonic::Status> {
        let response = match Daemon::leave_org(self.inner.clone()).await {
            Ok(_) => CommonResponse {
                success: true,
                message: String::from("OK"),
            },
            Err(message) => CommonResponse {
                success: false,
                message,
            },
        };

        Ok(tonic::Response::from(response))
    }
}
