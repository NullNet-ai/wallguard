use crate::arguments::Arguments;
use crate::reverse_tunnel::ReverseTunnel;
use crate::token_provider::TokenProvider;
use clap::Parser;
use nullnet_libwallguard::WallGuardGrpcInterface;

#[derive(Clone, Debug)]
pub struct AppContext {
    pub arguments: Arguments,
    pub token_provider: TokenProvider,
    pub server: WallGuardGrpcInterface,
    pub tunnel: ReverseTunnel,
}

impl AppContext {
    pub async fn new() -> Self {
        let arguments = match Arguments::try_parse() {
            Ok(args) => args,
            Err(err) => {
                log::error!("Failed to parse CLI arguments: {}", err);
                std::process::exit(1);
            }
        };

        let token_provider = TokenProvider::new();

        let server = WallGuardGrpcInterface::new(&arguments.addr, arguments.port).await;

        let tunnel = ReverseTunnel::new(&arguments.tunnel_addr, arguments.tunnel_port).unwrap();

        Self {
            arguments,
            token_provider,
            server,
            tunnel,
        }
    }
}
