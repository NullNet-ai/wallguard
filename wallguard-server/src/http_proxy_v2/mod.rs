use nullnet_liberror::{Error, ErrorHandler, Location, location};
use pingora::{proxy::http_proxy_service, server::Server};

use crate::app_context::AppContext;

mod config;
mod connector;
mod proxy;

pub async fn run_http_proxy(context: AppContext) -> Result<(), Error> {
    let config = config::HttpProxyConfig::from_env();

    let mut server = Server::new(None).handle_err(location!())?;

    server.bootstrap();

    let mut service = http_proxy_service(&server.configuration, proxy::Proxy::new(context));

    service.add_tcp(config.addr.to_string().as_str());

    server.add_service(service);

    let _ = tokio::task::spawn_blocking(move || server.run_forever()).await;

    Ok(())
}
