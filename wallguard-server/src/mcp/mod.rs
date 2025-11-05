use crate::{
    app_context::AppContext,
    mcp::{config::McpConfig, service::MCPService, session_manager::SessionManagerEx},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use rmcp::transport::streamable_http_server::{
    StreamableHttpService,
};

mod config;
mod middleware;
mod schema;
mod service;
mod session_manager;

pub async fn run_mcp_server(context: AppContext) -> Result<(), Error> {
    let cfg = McpConfig::from_env();

    let ctx = context.clone();
    let service = StreamableHttpService::new(
        move || Ok(MCPService::new(ctx.clone())),
        SessionManagerEx::new(context.clone()).into(),
        Default::default(),
    );

    let layer =
        axum::middleware::from_fn_with_state(context, middleware::authentication_middleware);
        
    let router = axum::Router::new()
        .nest_service("/mcp", service)
        .layer(layer);

    let listener = tokio::net::TcpListener::bind(cfg.addr)
        .await
        .handle_err(location!())?;

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { tokio::signal::ctrl_c().await.unwrap() })
        .await
        .handle_err(location!())
}
