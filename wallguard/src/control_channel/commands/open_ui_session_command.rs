use std::net::SocketAddr;

use nullnet_liberror::{location, Error, ErrorHandler, Location};
use tokio::net::TcpStream;
use wallguard_common::protobuf::wallguard_commands::UiSessionData;

use crate::{context::Context, control_channel::command::ExecutableCommand, utilities};

pub struct OpenUiSessionCommand {
    context: Context,
    data: UiSessionData,
}

impl OpenUiSessionCommand {
    pub fn new(context: Context, data: UiSessionData) -> Self {
        Self { context, data }
    }
}

impl ExecutableCommand for OpenUiSessionCommand {
    async fn execute(self) -> Result<(), Error> {
        log::debug!("Received OpenUiSessionCommand");

        let addr: SocketAddr = match self.data.protocol.to_lowercase().as_str() {
            "http" => "127.0.0.1:80".parse().unwrap(),
            "https" => "127.0.0.1:443".parse().unwrap(),
            _ => {
                return Err(format!("Unsupported protocol: {}", self.data.protocol))
                    .handle_err(location!())
            }
        };

        let local_stream = TcpStream::connect(addr).await.handle_err(location!())?;

        let Ok(tunnel) = self
            .context
            .tunnel
            .request_channel(&self.data.tunnel_token)
            .await
        else {
            return Err("Cant establish tunnel connection").handle_err(location!());
        };

        tokio::spawn(async move {
            let _ = utilities::io::copy_bidirectional_for_tunnel(tunnel, local_stream).await;
        });

        Ok(())
    }
}
