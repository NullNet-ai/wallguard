mod session;

use nullnet_libconfmon::Platform;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use session::RemoteAccessSession;
use std::net::SocketAddr;

pub struct RemoteAccessManager {
    shell_session: Option<RemoteAccessSession>,
    ui_session: Option<RemoteAccessSession>,

    platform: Platform,
    server_addr: SocketAddr,
}

impl RemoteAccessManager {
    pub fn new(platform: Platform, server_addr: SocketAddr) -> Self {
        Self {
            shell_session: None,
            ui_session: None,
            platform,
            server_addr,
        }
    }

    pub async fn start_tty_session(&mut self, tunnel_id: String) -> Result<(), Error> {
        if self.shell_session.is_some() {
            return Err("Session already in progress").handle_err(location!());
        }

        self.shell_session = Some(RemoteAccessSession::tty(
            tunnel_id,
            self.server_addr,
            self.platform,
        ));

        log::debug!("Started Shell r.a. session");

        Ok(())
    }

    pub async fn start_ui_session(
        &mut self,
        tunnel_id: String,
        protocol: &str,
    ) -> Result<(), Error> {
        if self.ui_session.is_some() {
            return Err("Session already in progress").handle_err(location!());
        }

        self.ui_session = Some(RemoteAccessSession::ui(
            tunnel_id,
            protocol,
            self.server_addr,
            self.platform,
        )?);

        log::debug!("Started UI r.a. session");

        Ok(())
    }

    pub async fn terminate_ui_session(&mut self) -> Result<(), Error> {
        log::debug!("Terminating UI r.a. session");

        match self.ui_session.take() {
            Some(session) => {
                session.terminate().await;
                Ok(())
            }
            None => Err("No session in progress").handle_err(location!()),
        }
    }

    pub async fn terminate_shell_session(&mut self) -> Result<(), Error> {
        log::debug!("Terminating Shell r.a. session");

        match self.shell_session.take() {
            Some(session) => {
                session.terminate().await;
                Ok(())
            }
            None => Err("No session in progress").handle_err(location!()),
        }
    }

    pub fn has_ui_session(&mut self) -> bool {
        self.ui_session.is_some()
    }

    pub fn has_shell_session(&mut self) -> bool {
        self.shell_session.is_some()
    }
}
