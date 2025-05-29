#[rustfmt::skip]
mod wallguard_cli;
use std::net::SocketAddr;

use crate::app_context::AppContext;
use crate::cli::wallguard_cli::status::State;
use crate::cli::wallguard_cli::wallguard_cli_server::WallguardCli;
use crate::cli::wallguard_cli::wallguard_cli_server::WallguardCliServer;
use crate::cli::wallguard_cli::Caps;
// use crate::cli::wallguard_cli::Connected;
use crate::cli::wallguard_cli::Empty;
// use crate::cli::wallguard_cli::Error;
use crate::cli::wallguard_cli::Idle;
use crate::cli::wallguard_cli::JoinOrgReq;
use crate::cli::wallguard_cli::JoinOrgRes;
use crate::cli::wallguard_cli::LeaveOrgRes;
use crate::cli::wallguard_cli::Status;

#[derive(Debug)]
pub struct CliServer {
    context: AppContext,
}

impl CliServer {
    pub fn new(context: AppContext) -> Self {
        Self { context }
    }

    pub async fn run(self) -> Result<(), tonic::transport::Error> {
        let server = WallguardCliServer::new(self);
        let addr: SocketAddr = "127.0.0.1:54056".parse().unwrap();
        
        tonic::transport::Server::builder()
            .add_service(server)
            .serve(addr)
            .await
    }
}

type TReq<T> = tonic::Request<T>;
type TRes<T> = Result<tonic::Response<T>, tonic::Status>;

#[tonic::async_trait]
impl WallguardCli for CliServer {
    async fn get_status(&self, request: TReq<Empty>) -> TRes<Status> {
        let status = Status {
            state: Some(State::Idle(Idle {
                message: String::from("Test Message"),
            })),
        };

        let response = tonic::Response::from(status);
        Ok(response)
    }

    async fn get_capabilities(&self, request: TReq<Empty>) -> TRes<Caps> {
        let caps = Caps {
            traffic: true,
            telemetry: false,
            sysconfig: false,
        };

        let response = tonic::Response::from(caps);
        Ok(response)
    }

    async fn join_org(&self, request: TReq<JoinOrgReq>) -> TRes<JoinOrgRes> {
        let res = JoinOrgRes {
            success: false,
            message: String::from("not implemented"),
        };

        let response = tonic::Response::from(res);
        Ok(response)
    }

    async fn leave_org(&self, request: TReq<Empty>) -> TRes<LeaveOrgRes> {
        let res = LeaveOrgRes {
            success: false,
            message: String::from("not implemented"),
        };
        let response = tonic::Response::from(res);
        Ok(response)
    }
}
