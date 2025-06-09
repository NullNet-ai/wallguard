use crate::{arguments::Arguments, daemon::Daemon, storage::Storage};
use clap::Parser as _;

mod arguments;
mod context;
mod control_channel;
mod daemon;
mod data_transmission;
mod device_uuid;
mod pty;
mod reverse_tunnel;
mod storage;
mod timer;
mod token_provider;
mod utilities;
mod constants;

#[tokio::main]
async fn main() {
    env_logger::init();

    let arguments = match Arguments::try_parse() {
        Ok(args) => args,
        Err(err) => {
            log::error!("Failed to parse CLI arguments: {}", err);
            std::process::exit(1);
        }
    };

    Storage::init().await.unwrap();

    if !nix::unistd::Uid::effective().is_root() {
        log::error!("This program must be run as root. Exiting ...");
        std::process::exit(-1);
    }

    let Some(device_uuid) = device_uuid::retrieve_device_uuid() else {
        log::error!("Failed to retrieve device UUID, exiting ...");
        std::process::exit(-1);
    };

    Daemon::run(device_uuid, arguments).await.unwrap()
}
