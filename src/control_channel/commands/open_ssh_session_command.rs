use crate::app_context::AppContext;
use crate::control_channel::command::ExecutableCommand;
use crate::utilities;
use nullnet_liberror::{location, ErrorHandler, Location};
use nullnet_libwallguard::SshSessionData;
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;

pub struct OpenSshSessionCommand {
    context: AppContext,
    data: SshSessionData,
}

impl ExecutableCommand for OpenSshSessionCommand {
    async fn execute(self) -> Result<(), nullnet_liberror::Error> {
        log::debug!("Received OpenSshSessionCommand");

        if let Err(err) = utilities::ssh::add_ssh_key_if_missing(&self.data.public_key).await {
            log::error!("Failed authorize public key: {}", err);
            return Err(err).handle_err(location!());
        }

        let Ok(sshd_stream) = TcpStream::connect("127.0.0.1:22").await else {
            return Err("Cant establish sshd connection").handle_err(location!());
        };

        let Ok(tunnel_stream) = self
            .context
            .tunnel
            .request_channel(&self.data.tunnel_token)
            .await
        else {
            return Err("Cant establish tunnel connection").handle_err(location!());
        };

        tokio::spawn(async move {
            let mut s1 = tunnel_stream;
            let mut s2 = sshd_stream;
            let _ = copy_bidirectional(&mut s1, &mut s2).await;
        });

        Ok(())
    }
}

impl OpenSshSessionCommand {
    pub fn new(context: AppContext, data: SshSessionData) -> Self {
        Self { context, data }
    }
}
