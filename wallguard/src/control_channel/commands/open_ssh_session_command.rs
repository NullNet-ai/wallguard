use crate::context::Context;
use crate::control_channel::command::ExecutableCommand;
use crate::utilities;
use nullnet_liberror::{location, ErrorHandler, Location};
use tokio::io::copy_bidirectional;
use tokio::io::AsyncWriteExt as _;
use tokio::net::TcpStream;
use wallguard_common::protobuf::wallguard_commands::SshSessionData;

pub struct OpenSshSessionCommand {
    context: Context,
    data: SshSessionData,
}

impl ExecutableCommand for OpenSshSessionCommand {
    async fn execute(self) -> Result<(), nullnet_liberror::Error> {
        log::debug!("Received OpenSshSessionCommand");

        if let Err(err) = utilities::ssh::add_ssh_key_if_missing(&self.data.public_key).await {
            log::error!("Failed to authorize public key: {err}");
            return Err(err).handle_err(location!());
        }

        let ports = match utilities::ssh::get_sshd_ports_from_sshd_t().await {
            Ok(values) if !values.is_empty() => values,
            Ok(_) => {
                log::error!("No SSHD ports found in configuration");
                return Err("No ports found").handle_err(location!());
            }
            Err(err) => {
                log::error!("Failed to get sshd port: {err}");
                return Err(err).handle_err(location!());
            }
        };

        let Ok(sshd_stream) = TcpStream::connect(format!("127.0.0.1:{}", ports[0])).await else {
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

            let _ = s1.shutdown().await;
            let _ = s2.shutdown().await;
        });

        Ok(())
    }
}

impl OpenSshSessionCommand {
    pub fn new(context: Context, data: SshSessionData) -> Self {
        Self { context, data }
    }
}
