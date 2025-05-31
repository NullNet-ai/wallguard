#[rustfmt::skip]
mod wallguard_cli;
mod authorization_task;
mod cli_server;
mod state;

use crate::daemon::authorization_task::AuthorizationTask;
use crate::daemon::cli_server::CliServer;
use crate::daemon::state::DaemonState;
use crate::storage::{Secret, Storage};
use crate::utilities;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use wallguard_cli::wallguard_cli_server::WallguardCliServer;
use wallguard_cli::Status;

#[derive(Debug, Default)]
pub struct Daemon {
    state: DaemonState,
}

impl Daemon {
    pub async fn run() -> Result<(), Error> {
        let daemon = Arc::new(Mutex::new(Daemon::default()));

        if let Some(org_id) = Storage::get_value(Secret::ORG_ID) {
            log::info!("Found org id {org_id}, attempting to connect");
            let _ = Daemon::join_org(daemon.clone(), org_id).await;
        } else {
            log::info!("No org ID, entering idle state");
        }

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

    pub(crate) async fn join_org(this: Arc<Mutex<Daemon>>, org_id: String) -> Result<(), String> {
        let mut lock = this.lock().await;
        match &lock.state {
            DaemonState::Idle(_) => {
                Storage::set_value(Secret::ORG_ID, &org_id)
                    .map_err(|err| err.to_str().to_string())?;

                let task = AuthorizationTask::new(this.clone());
                task.run();

                lock.state = DaemonState::Authorization(task);
                Ok(())
            }
            _ => Err(format!(
                "Can not join a new organization from the current state: {}",
                lock.state
            )),
        }
    }

    pub(crate) async fn leave_org(this: Arc<Mutex<Daemon>>) -> Result<(), String> {
        let mut this = this.lock().await;

        match &this.state {
            DaemonState::Connected(_, _) => {
                let timestamp = utilities::time::timestamp();
                // @TODO: perform actual reset
                this.state = DaemonState::Idle(timestamp as u64);
                Ok(())
            }
            DaemonState::Authorization(task) => {
                task.shutdown();
                let timestamp = utilities::time::timestamp();
                this.state = DaemonState::Idle(timestamp as u64);
                Ok(())
            }
            _ => Err(format!(
                "Can not leave current organization from the current state: {}",
                this.state
            )),
        }
    }

    pub(crate) async fn on_authorized(this: Arc<Mutex<Daemon>>) {
        let mut lock = this.lock().await;
        // Establish control stream and spawn a task
        // let task = ConnectionTask::new();
        lock.state = DaemonState::Connected(1, String::new());
    }
}
