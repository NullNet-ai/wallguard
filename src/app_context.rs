use crate::{cli, reverse_tunnel::ReverseTunnel, token_provider::TokenProvider};
use clap::Parser;
use nullnet_libwallguard::WallGuardGrpcInterface;

#[derive(Clone)]
pub struct AppContext {
    pub arguments: cli::Args,
    pub token_provider: TokenProvider,
    pub server: WallGuardGrpcInterface,
    pub tunnel: ReverseTunnel,
}

impl AppContext {
    pub async fn new() -> Self {
        let arguments = match cli::Args::try_parse() {
            Ok(args) => args,
            Err(err) => {
                log::error!("Failed to parse CLI arguments: {}", err);
                std::process::exit(1);
            }
        };

        if let Err(err) = arguments.validate() {
            log::error!("{}", err);
            std::process::exit(1);
        }

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
