use crate::app_context::AppContext;
use crate::control_service::run_control_service;
use crate::http_api::run_http_api;
use crate::http_proxy_v2::run_http_proxy;
use crate::mcp::run_mcp_server;
use crate::reverse_tunnel::run_tunnel_acceptor;
use nullnet_liberror::Error;

pub async fn linux_main() {
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
        .expect("Failed to acquire SYSDEV token, check the credentials");

    prepare_records(&app_context)
        .await
        .expect("Failed to prepare records");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = run_control_service(app_context.clone()) => {},
        _ = run_http_api(app_context.clone()) => {},
        _ = run_mcp_server(app_context.clone()) => {},
        _ = run_http_proxy(app_context.clone()) => {},
        _ = run_tunnel_acceptor(app_context.clone()) => {},
    }
}

async fn prepare_records(context: &AppContext) -> Result<(), Error> {
    let token = context.sysdev_token_provider.get().await?;

    context
        .datastore
        .update_all_devices_online_status(&token.jwt, false, false)
        .await?;

    context
        .datastore
        .delete_all_device_instances(&token.jwt, false)
        .await?;

    Ok(())
}
