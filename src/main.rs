use app_context::AppContext;
use control_channel::ControlChannel;

mod app_context;
mod cli;
mod control_channel;
mod reverse_tunnel;
mod token_provider;
mod utilities;

#[tokio::main]
async fn main() {
    env_logger::init();
    let context = AppContext::new().await;
    ControlChannel::new(context).run().await.unwrap();
}
