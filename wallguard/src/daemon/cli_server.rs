use super::wallguard_cli::wallguard_cli_server::WallguardCli;
use super::wallguard_cli::Caps;
use super::wallguard_cli::Empty;
use super::wallguard_cli::JoinOrgReq;
use super::wallguard_cli::JoinOrgRes;
use super::wallguard_cli::LeaveOrgRes;
use super::wallguard_cli::Status;
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
        _: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Status>, tonic::Status> {
        let status = self.inner.lock().await.get_status();
        Ok(tonic::Response::from(status))
    }

    async fn get_capabilities(
        &self,
        _: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Caps>, tonic::Status> {
        let caps = Caps {
            traffic: false,
            telemetry: false,
            sysconfig: false,
        };

        let response = tonic::Response::from(caps);
        Ok(response)
    }

    async fn join_org(
        &self,
        request: tonic::Request<JoinOrgReq>,
    ) -> Result<tonic::Response<JoinOrgRes>, tonic::Status> {
        let org_id = request.into_inner().org_id;

        let response = match Daemon::join_org(self.inner.clone(), org_id).await {
            Ok(_) => JoinOrgRes {
                success: true,
                message: String::from("OK"),
            },
            Err(message) => JoinOrgRes {
                success: false,
                message,
            },
        };

        Ok(tonic::Response::from(response))
    }

    async fn leave_org(
        &self,
        _: tonic::Request<Empty>,
    ) -> Result<tonic::Response<LeaveOrgRes>, tonic::Status> {
        let response = match Daemon::leave_org(self.inner.clone()).await {
            Ok(_) => LeaveOrgRes {
                success: true,
                message: String::from("OK"),
            },
            Err(message) => LeaveOrgRes {
                success: false,
                message,
            },
        };

        Ok(tonic::Response::from(response))
    }
}
