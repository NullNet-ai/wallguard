use std::time::Duration;

use libwallguard::WallGuardGrpcInterface;
use log::Level;

use crate::authentication::AuthHandler;
use crate::cli::Args;
use crate::logger::Logger;

pub async fn routine(auth: AuthHandler, args: Args) {
    let interval = Duration::from_secs(args.heartbeat_interval);
    loop {
        match auth.obtain_token_safe().await {
            Ok(token) => {
                let mut client = WallGuardGrpcInterface::new(&args.addr, args.port).await;

                match client.heartbeat(token).await {
                    Ok(response) => {
                        if !response.success {
                            Logger::log(
                                Level::Error,
                                format!("Heartbeat: Request failed failed - {}", response.message),
                            )
                        }
                    }
                    Err(msg) => Logger::log(
                        Level::Error,
                        format!("Heartbeat: Request failed failed - {}", msg),
                    ),
                }
            }
            Err(msg) => Logger::log(
                Level::Error,
                format!("Heartbeat: Authentication failed - {}", msg),
            ),
        };

        tokio::time::sleep(interval).await;
    }
}
