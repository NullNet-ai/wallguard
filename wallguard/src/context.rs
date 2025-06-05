use std::sync::Arc;

use crate::arguments::Arguments;
use crate::daemon::Daemon;
use crate::reverse_tunnel::ReverseTunnel;
use crate::token_provider::TokenProvider;
use nullnet_liberror::Error;
use nullnet_libwallguard::WallGuardGrpcInterface;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Context {
    pub token_provider: TokenProvider,
    pub server: WallGuardGrpcInterface,
    pub tunnel: ReverseTunnel,
    pub daemon: Arc<Mutex<Daemon>>,
}

impl Context {
    pub async fn new(arguments: Arguments, daemon: Arc<Mutex<Daemon>>) -> Result<Self, Error> {
        let token_provider = TokenProvider::new();

        let server = WallGuardGrpcInterface::new(&arguments.addr, arguments.port).await?;

        let tunnel = ReverseTunnel::new(&arguments.tunnel_addr, arguments.tunnel_port).unwrap();

        Ok(Self {
            token_provider,
            server,
            tunnel,
            daemon,
        })
    }
}
