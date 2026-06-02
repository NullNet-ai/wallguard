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
mod netinfo;
mod pty;
mod reverse_tunnel;
mod server_data;
mod storage;
mod timer;
mod token_provider;
mod utilities;
mod wg_server;

mod remote_desktop;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn check_privileges() {
    #[cfg(windows)]
    {
        if !is_elevated::is_elevated() {
            println!("This program must be run as Administrator. Exiting …");
            std::process::exit(-1);
        }
    }

    #[cfg(target_os = "linux")]
    {
        use caps::{CapSet, Capability};
        let has_net_raw =
            caps::has_cap(None, CapSet::Effective, Capability::CAP_NET_RAW).unwrap_or(false);
        let has_net_admin =
            caps::has_cap(None, CapSet::Effective, Capability::CAP_NET_ADMIN).unwrap_or(false);
        if !has_net_raw || !has_net_admin {
            println!(
                "wallguard requires CAP_NET_RAW and CAP_NET_ADMIN capabilities. Run:\n  \
                 sudo setcap cap_net_raw,cap_net_admin+eip /usr/local/bin/wallguard"
            );
            std::process::exit(-1);
        }
    }

    #[cfg(all(unix, not(target_os = "linux")))]
    {
        if !nix::unistd::Uid::effective().is_root() {
            println!("This program must be run as root. Exiting ...");
            std::process::exit(-1);
        }
    }
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    check_privileges();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

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
