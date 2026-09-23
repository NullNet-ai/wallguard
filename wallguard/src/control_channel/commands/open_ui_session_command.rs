use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::net::SocketAddr;
use std::time::Duration;
use wallguard_common::protobuf::wallguard_commands::UiSessionData;

use crate::utilities::net;
use crate::{context::Context, control_channel::command::ExecutableCommand};

/// Each UI tunnel carries one pooled upstream HTTP connection from the
/// server's proxy. One that has carried nothing for this long is an
/// abandoned keep-alive connection, not an active page.
const UI_IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

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

        // The address comes from service discovery and is the listener's
        // bind address, e.g. `::` for a dual-stack wildcard listener.
        let local_stream = net::connect_local(addr).await.handle_err(location!())?;

        let Ok(tunnel) = self
            .context
            .tunnel
            .request_channel(&self.data.tunnel_token)
            .await
        else {
            return Err("Cant establish tunnel connection").handle_err(location!());
        };

        tokio::spawn(async move {
            net::relay(tunnel, local_stream, Some(UI_IDLE_TIMEOUT)).await;
        });

        Ok(())
    }
}
