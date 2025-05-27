use std::net::SocketAddr;

use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::UiSessionData;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{app_context::AppContext, control_channel::command::ExecutableCommand};

pub struct OpenUiSessionCommand {
    context: AppContext,
    data: UiSessionData,
}

impl OpenUiSessionCommand {
    pub fn new(context: AppContext, data: UiSessionData) -> Self {
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

        let Ok(remote_stream) = self
            .context
            .tunnel
            .request_channel(&self.data.tunnel_token)
            .await
        else {
            return Err("Cant establish tunnel connection").handle_err(location!());
        };

        tokio::spawn(async move {
            let mut s1 = local_stream;
            let mut s2 = remote_stream;
            
            let _ = tokio::io::copy_bidirectional(&mut s1, &mut s2).await;

            let _ = s1.shutdown().await;
            let _ = s2.shutdown().await;
        });
        
        Ok(())
    }
}
