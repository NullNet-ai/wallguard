#[rustfmt::skip]
mod wallguard_cli;
mod cli_server;
mod state;

use crate::daemon::cli_server::CliServer;
use crate::daemon::state::DaemonState;
use crate::utilities;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use wallguard_cli::status::State;
use wallguard_cli::wallguard_cli_server::WallguardCli;
use wallguard_cli::wallguard_cli_server::WallguardCliServer;
use wallguard_cli::Caps;
use wallguard_cli::Empty;
use wallguard_cli::Idle;
use wallguard_cli::JoinOrgReq;
use wallguard_cli::JoinOrgRes;
use wallguard_cli::LeaveOrgRes;
use wallguard_cli::Status;

#[derive(Debug, Default)]
pub struct Daemon {
    state: DaemonState,
}

impl Daemon {
    pub async fn run() -> Result<(), Error> {
        let daemon = Arc::new(Mutex::new(Daemon::default()));

        let addr: SocketAddr = "127.0.0.1:54056".parse().unwrap();
        let cli_server = WallguardCliServer::new(CliServer::from(daemon));

        tonic::transport::Server::builder()
            .add_service(cli_server)
            .serve(addr)
            .await
            .handle_err(location!())
    }

    pub fn get_status(&self) -> Status {
        self.state.clone().into()
    }

    pub async fn join_org(&mut self, org_id: String) -> Result<(), String> {
        match &self.state {
            DaemonState::Idle(_) => {
                let timestamp = utilities::time::timestamp();
                // @TODO: perform actual connection attempt
                self.state = DaemonState::Authorization(timestamp as u64, org_id);
                Ok(())
            }
            _ => Err(format!(
                "Can not join a new organization from the current state. {}",
                self.state
            )),
        }
    }

    pub async fn leave_org(&mut self) -> Result<(), String> {
        match &self.state {
            DaemonState::Connected(_, _) | DaemonState::Authorization(_, _) => {
                let timestamp = utilities::time::timestamp();
                // @TODO: perform actual reset
                self.state = DaemonState::Idle(timestamp as u64);
                Ok(())
            }
            _ => Err(format!(
                "Can not leave current organization from the current state. {}",
                self.state
            )),
        }
    }
}
