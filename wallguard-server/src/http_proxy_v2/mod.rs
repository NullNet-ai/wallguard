use nullnet_liberror::{Error, ErrorHandler, Location, location};
use pingora::{proxy::http_proxy_service, server::Server};

use crate::app_context::AppContext;

mod connector;
mod proxy;

pub async fn run_http_proxy(context: AppContext) -> Result<(), Error> {
    let mut server = Server::new(None).handle_err(location!())?;

    server.bootstrap();

    let mut service = http_proxy_service(&server.configuration, proxy::Proxy::new(context));

    let addr_str = format!("0.0.0.0:{}", 16);

    service.add_tcp(addr_str.as_str());
    server.add_service(service);

    let _ = tokio::task::spawn_blocking(move || server.run_forever()).await;

    Ok(())
}
