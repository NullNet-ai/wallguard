use app_context::AppContext;
use control_service::run_control_service;
use http_proxy::run_http_proxy;
use mcp::run_mcp_server;

mod app_context;
mod control_service;
mod datastore;
mod http_proxy;
mod http_proxy_v2;
mod mcp;
mod orchestrator;
mod reverse_tunnel;
mod token_provider;
mod traffic_handler;
mod utilities;

#[tokio::main]
async fn main() {
    env_logger::init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let app_context = AppContext::new().await.unwrap_or_else(|err| {
        log::error!("Failed to initialize application context: {}", err.to_str());
        std::process::exit(1);
    });

    app_context
        .root_token_provider
        .get()
        .await
        .expect("Failed to acquire ROOT token, check the credentials");

    app_context
        .sysdev_token_provider
        .get()
        .await
        .expect("Faield to acquire SYSDEV token, check the credentials");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = run_control_service(app_context.clone()) => {},
        _ = run_http_proxy(app_context.clone()) => {},
        _ = run_mcp_server(app_context.clone()) => {}
    }
}
