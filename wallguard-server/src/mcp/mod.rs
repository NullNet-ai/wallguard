use crate::{
    app_context::AppContext,
    mcp::{config::McpConfig, service::MCPService},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};

mod config;
mod schema;
mod service;

pub async fn run_mcp_server(context: AppContext) -> Result<(), Error> {
    let cfg = McpConfig::from_env();

    let ctx = context.clone();
    let service = StreamableHttpService::new(
        move || Ok(MCPService::new(ctx.clone())),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);

    let listener = tokio::net::TcpListener::bind(cfg.addr)
        .await
        .handle_err(location!())?;

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { tokio::signal::ctrl_c().await.unwrap() })
        .await
        .handle_err(location!())
}
