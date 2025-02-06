use std::time::Duration;

use libwallguard::WallGuardGrpcInterface;

use crate::authentication::AuthHandler;
use crate::cli::Args;

pub async fn routine(auth: AuthHandler, args: Args) {
    if cfg!(feature = "no-datastore") {
        println!("Datastore functionality is disabled. Stopping heartbeat routine ...");
        return;
    }

    let interval = Duration::from_secs(args.heartbeat_interval);
    loop {
        match auth.obtain_token_safe().await {
            Ok(token) => {
                let mut client = WallGuardGrpcInterface::new(&args.addr, args.port).await;

                match client.heartbeat(token).await {
                    Ok(response) => {
                        if !response.success {
                            println!("Heartbeat: Request failed failed - {}", response.message)
                        }
                    }
                    Err(msg) => {
                        println!("Heartbeat: Request failed failed - {}", msg)
                    }
                }
            }
            Err(msg) => println!("Heartbeat: Authentication failed - {}", msg),
        };

        tokio::time::sleep(interval).await;
    }
}
