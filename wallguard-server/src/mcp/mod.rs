use crate::{
    app_context::AppContext,
    mcp::{config::McpConfig, service::MCPService},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use tokio_util::sync::CancellationToken;

mod config;
mod middleware;
mod schema;
mod service;

pub async fn run_mcp_server(context: AppContext) -> Result<(), Error> {
    let cfg = McpConfig::from_env();

    let config = SseServerConfig {
        bind: cfg.addr,
        sse_path: "/sse".into(),
        post_path: "/message".into(),
        ct: CancellationToken::new(),
        sse_keep_alive: None,
    };

    let (sse_server, mut router) = SseServer::new(config);

    let listener = tokio::net::TcpListener::bind(sse_server.config.bind)
        .await
        .handle_err(location!())?;

    let ct = sse_server.config.ct.child_token();

    let service = MCPService::new(context.clone());
    sse_server.with_service(move || service.clone());

    router = router.layer(axum::middleware::from_fn_with_state(
        context,
        middleware::authentication_middleware,
    ));

    log::info!("MCP server is running on {}", cfg.addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { ct.cancelled().await })
        .await
        .handle_err(location!())
}
