use control_service::{run_control_service, ControlServiceConfig};

mod control_service;
mod orchestrator;
mod protocol;

#[tokio::main]
async fn main() {
    let csconf = ControlServiceConfig::from_env().unwrap_or_else(|err| {
        log::error!(
            "Failed to construct `ControlServiceConfig` from environment: {}",
            err
        );
        std::process::exit(1);
    });

    run_control_service(csconf).await
}
