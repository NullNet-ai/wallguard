use app_context::AppContext;
use control_channel::ControlChannel;

mod app_context;
mod cli;
mod control_channel;
mod device_uuid;
mod pty;
mod reverse_tunnel;
mod token_provider;
mod utilities;

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

    println!("Device UUID: {}", device_uuid);

    // let context = AppContext::new().await;
    // ControlChannel::new(context).run().await.unwrap();
}
