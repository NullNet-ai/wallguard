use app_context::AppContext;
use control_channel::ControlChannel;

use crate::cli::CliServer;

mod app_context;
mod cli;
mod control_channel;
mod device_uuid;
mod pty;
mod reverse_tunnel;
mod token_provider;
mod utilities;
mod arguments;

#[tokio::main]
async fn main() {
    env_logger::init();

    if !nix::unistd::Uid::effective().is_root() {
        log::error!("This program must be run as root. Exiting ...");
        std::process::exit(-1);
    }

    let Some(device_uuid) = device_uuid::retrieve_device_uuid() else {
        log::error!("Failed to retrieve device UUID, exiting ...");
        std::process::exit(-1);
    };


    let context = AppContext::new().await;
    
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        retval = CliServer::new(context).run() => {
            if let Err(err) = retval {
                log::error!("CLI server failed: {}", err);
            }
        }
    }
    
    // ControlChannel::new(context).run().await.unwrap();
}
