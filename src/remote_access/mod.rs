mod session;

use nullnet_libconfmon::Platform;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use session::RemoteAccessSession;
use std::net::SocketAddr;

pub struct RemoteAccessManager {
    session: Option<RemoteAccessSession>,
    platform: Platform,
    server_addr: SocketAddr,
}

impl RemoteAccessManager {
    pub fn new(platform: Platform, server_addr: SocketAddr) -> Self {
        Self {
            session: None,
            platform,
            server_addr,
        }
    }

    pub async fn start_tty_session(&mut self, tunnel_id: String) -> Result<(), Error> {        
        if self.session.is_some() {
            return Err("Session already in progress").handle_err(location!());
        }

        self.session = Some(RemoteAccessSession::tty(
            tunnel_id,
            self.server_addr,
            self.platform,
        ));

        log::debug!("Started TTY r.a. session");

        Ok(())
    }

    pub async fn start_ui_session(
        &mut self,
        tunnel_id: String,
        protocol: &str,
    ) -> Result<(), Error> {
        if self.session.is_some() {
            return Err("Session already in progress").handle_err(location!());
        }

        self.session = Some(RemoteAccessSession::ui(
            tunnel_id,
            protocol,
            self.server_addr,
            self.platform,
        )?);

        log::debug!("Started UI r.a. session");

        Ok(())
    }

    pub async fn terminate(&mut self) -> Result<(), Error> {
        log::debug!("Terminating r.a. session");

        match self.session.take() {
            Some(session) => {
                session.terminate().await;
                Ok(())
            }
            None => Err("No session in progress").handle_err(location!()),
        }
    }

    pub fn has_session(&mut self) -> bool {
        self.session.is_some()
    }
}
