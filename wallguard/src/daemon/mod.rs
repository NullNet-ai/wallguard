#[rustfmt::skip]
mod wallguard_cli;
mod authorization_task;
mod cli_server;
mod state;

use crate::arguments::Arguments;
use crate::context::Context;
use crate::daemon::authorization_task::AuthorizationTask;
use crate::daemon::cli_server::CliServer;
use crate::daemon::state::DaemonState;
use crate::storage::{Secret, Storage};
use crate::utilities;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::AuthorizationApproved;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use wallguard_cli::wallguard_cli_server::WallguardCliServer;
use wallguard_cli::Status;

#[derive(Debug, Default)]
pub struct Daemon {
    uuid: String,
    arguments: Arguments,
    state: DaemonState,
}

impl Daemon {
    pub async fn run(uuid: impl Into<String>, arguments: Arguments) -> Result<(), Error> {
        let daemon = Arc::new(Mutex::new(Daemon {
            uuid: uuid.into(),
            arguments,
            state: DaemonState::default(),
        }));

        if let Some(org_id) = Storage::get_value(Secret::ORG_ID).await {
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

    pub(crate) fn get_status(&self) -> Status {
        self.state.clone().into()
    }

    pub(crate) async fn join_org(this: Arc<Mutex<Daemon>>, org_id: String) -> Result<(), String> {
        let mut lock = this.lock().await;
        match &lock.state {
            DaemonState::Idle(_) => {
                Storage::set_value(Secret::ORG_ID, &org_id)
                    .await
                    .map_err(|err| err.to_str().to_string())?;

                let context = Context::new(lock.arguments.clone())
                    .await
                    .map_err(|err| err.to_str().to_string())?;

                lock.state = DaemonState::Authorization(AuthorizationTask::new(
                    this.clone(),
                    context,
                    org_id,
                ));
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
                Storage::delete_value(Secret::ORG_ID)
                    .await
                    .map_err(|err| err.to_str().to_string())?;

                // @TODO: Cancel control stream

                let timestamp = utilities::time::timestamp();
                this.state = DaemonState::Idle(timestamp as u64);
                Ok(())
            }
            DaemonState::Authorization(task) => {
                task.shutdown();
                let timestamp = utilities::time::timestamp();
                this.state = DaemonState::Idle(timestamp as u64);
                Ok(())
            }
            DaemonState::Error(_, _) => {
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

    pub(crate) async fn get_uuid(this: Arc<Mutex<Daemon>>) -> String {
        this.lock().await.uuid.clone()
    }

    pub(crate) async fn on_authorized(this: Arc<Mutex<Daemon>>, data: AuthorizationApproved) {
        if data.app_id.is_some() {
            let _ = Storage::set_value(Secret::APP_ID, data.app_id()).await;
        }

        if data.app_secret.is_some() {
            let _ = Storage::set_value(Secret::APP_SECRET, data.app_secret()).await;
        }

        let app_id = Storage::get_value(Secret::APP_ID).await;
        let app_secret = Storage::get_value(Secret::APP_SECRET).await;

        if app_id.is_none() || app_secret.is_none() {
            let error = format!(
                "Couldn't find APP_ID or APP_SECRET, even though authorization has been approved"
            );
            Self::on_error(this, error).await;
        } else {
            // @TODO : Establish actual control stream etc
            log::debug!("Authorized!");

            this.lock().await.state = DaemonState::Connected(1, String::new());
        }
    }

    pub(crate) async fn on_error(this: Arc<Mutex<Daemon>>, reason: impl Into<String>) {
        let mut lock = this.lock().await;
        let timestamp = utilities::time::timestamp();
        lock.state = DaemonState::Error(timestamp as u64, reason.into());
    }
}
