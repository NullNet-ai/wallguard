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

fn init_logger() {
    #[cfg(unix)]
    let log_dir = std::path::PathBuf::from("/var/log");
    #[cfg(windows)]
    let log_dir = {
        let base = std::env::var("PROGRAMDATA").unwrap_or_else(|_| r"C:\ProgramData".to_string());
        std::path::PathBuf::from(base).join("wallguard")
    };

    use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming};
    Logger::try_with_env_or_str("info")
        .expect("Failed to configure logger")
        .log_to_file(
            FileSpec::default()
                .directory(log_dir)
                .basename("wallguard")
                .suffix("log")
                .suppress_timestamp(),
        )
        .rotate(
            Criterion::Size(10 * 1024 * 1024), // rotate at 10 MiB
            Naming::Timestamps,
            Cleanup::KeepLogFiles(5),
        )
        .duplicate_to_stderr(Duplicate::Error)
        .start()
        .expect("Failed to start logger");
}

/// Ensures this is the only running instance of the agent, regardless of
/// whether it was launched directly, via `wallguard-cli start`, or by the
/// OS service manager (systemd/launchd/rc.d/Windows service). Exits the
/// process if another instance already holds the lock.
fn acquire_single_instance_lock() -> wallguard_common::single_instance::InstanceLock {
    let lock_path = wallguard_common::single_instance::agent_lock_path();

    match wallguard_common::single_instance::InstanceLock::try_acquire(&lock_path) {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            log::error!("Another instance of the WallGuard agent is already running. Exiting.");
            std::process::exit(1);
        }
        Err(err) => {
            log::error!(
                "Failed to acquire single-instance lock at {}: {err}",
                lock_path.display()
            );
            std::process::exit(1);
        }
    }
}

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
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    check_privileges();
    init_logger();

    // Must happen before any other startup work (storage, network, org
    // join) so a duplicate instance never does anything besides exit.
    let _instance_lock = acquire_single_instance_lock();

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
