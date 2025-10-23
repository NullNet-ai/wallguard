use crate::arguments::Arguments;
use crate::client_data::ClientData;
use crate::daemon::Daemon;
use crate::server_data::ServerData;
use crate::storage::Storage;

use clap::Parser as _;

mod arguments;
mod client_data;
mod constants;
mod context;
mod control_channel;
mod daemon;
mod data_transmission;
mod fireparse;
mod pty;
mod remote_desktop;
mod reverse_tunnel;
mod server_data;
mod storage;
mod timer;
mod token_provider;
mod utilities;
mod wg_server;

fn check_privileges() {
    #[cfg(windows)]
    {
        if !is_elevated::is_elevated() {
            println!("This program must be run as Administrator. Exiting …");
            std::process::exit(-1);
        }
    }

    #[cfg(unix)]
    {
        if !nix::unistd::Uid::effective().is_root() {
            println!("This program must be run as root. Exiting ...");
            std::process::exit(-1);
        }
    }
}

#[tokio::main]
async fn main() {
    check_privileges();
    env_logger::init();

    let arguments = match Arguments::try_parse() {
        Ok(args) => args,
        Err(err) => {
            log::error!("Failed to parse CLI arguments: {err}");
            std::process::exit(1);
        }
    };

    Storage::init().await.unwrap();

    let Ok(server_data) = ServerData::try_from(&arguments) else {
        log::error!("Failed to collect server information. Exiting ...");
        std::process::exit(-1);
    };

    let Ok(client_data) = ClientData::try_from(arguments.platform) else {
        log::error!("Failed to collect client information. Exiting ...");
        std::process::exit(-1);
    };

    Daemon::run(client_data, server_data).await.unwrap()
}
