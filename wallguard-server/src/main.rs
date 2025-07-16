use app_context::AppContext;
use control_service::run_control_service;
use http_proxy::run_http_proxy;

mod app_context;
mod control_service;
mod datastore;
mod fireparse;
mod http_proxy;
mod orchestrator;
mod reverse_tunnel;
mod token_provider;
mod traffic_handler;
mod utilities;

#[tokio::main]
async fn main() {
    env_logger::init();

    let app_context = AppContext::new().await.unwrap_or_else(|err| {
        log::error!("Failed to initialize application context: {}", err.to_str());
        std::process::exit(1);
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = run_control_service(app_context.clone()) => {},
        _ = run_http_proxy(app_context) => {}
    }
}
