use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use wallguard_common::protobuf::wallguard_commands::UiSessionData;

use crate::{context::Context, control_channel::command::ExecutableCommand};

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

        let addr: SocketAddr = format!("{}:{}", self.data.local_addr, self.data.local_port)
            .parse()
            .handle_err(location!())?;

        let mut local_stream = TcpStream::connect(addr).await.handle_err(location!())?;

        let Ok(mut tunnel) = self
            .context
            .tunnel
            .request_channel(&self.data.tunnel_token)
            .await
        else {
            return Err("Cant establish tunnel connection").handle_err(location!());
        };

        tokio::spawn(async move {
            let _ = tokio::io::copy_bidirectional(&mut tunnel, &mut local_stream).await;
        });

        Ok(())
    }
}
