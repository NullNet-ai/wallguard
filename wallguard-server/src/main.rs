use app_context::AppContext;
use control_service::{ControlServiceConfig, run_control_service};

mod app_context;
mod control_service;
mod datastore;
mod orchestrator;
mod protocol;
mod token_provider;

#[tokio::main]
async fn main() {
    env_logger::init();

    let app_context = AppContext::new().await.unwrap_or_else(|err| {
        log::error!("Failed to initialize application context: {}", err.to_str());
        std::process::exit(1);
    });

    let csconf = ControlServiceConfig::from_env().unwrap_or_else(|err| {
        log::error!(
            "Failed to construct `ControlServiceConfig` from environment: {}",
            err
        );
        std::process::exit(1);
    });

    run_control_service(csconf, app_context).await
}
