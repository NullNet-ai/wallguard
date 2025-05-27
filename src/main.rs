use app_context::AppContext;
use control_channel::ControlChannel;
use devfingerprint::devfingerprint;

mod app_context;
mod cli;
mod control_channel;
mod devfingerprint;
mod pty;
mod reverse_tunnel;
mod token_provider;
mod utilities;

#[tokio::main]
async fn main() {
    env_logger::init();

    let Some(devfingerprint) = devfingerprint() else {
        log::error!("Failed to calculate device fingerprint, exiting ...");
        std::process::exit(-1);
    };

    println!("Device fingerprint: {}", devfingerprint);

    // let context = AppContext::new().await;
    // ControlChannel::new(context).run().await.unwrap();
}
