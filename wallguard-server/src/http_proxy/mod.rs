use crate::app_context::AppContext;
use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};
use api::request_session;
use config::HttpProxyConfig;

mod api;
mod config;
mod proxy;
mod ssh_gateway;
mod tty_gateway;
mod utilities;

pub async fn run_http_proxy(context: AppContext) {
    let config = HttpProxyConfig::from_env();
    log::info!("HTTP proxy listening on {}", config.addr);

    let context = web::Data::new(context);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "DELETE", "PUT"])
            .allowed_headers(vec![
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
            ])
            .max_age(3600);

        App::new()
            .app_data(context.clone())
            .wrap(cors)
            .route(
                "/wallguard/api/v1/remote_access",
                web::post().to(request_session),
            )
            .route(
                "/wallguard/gateway/ssh",
                web::to(ssh_gateway::open_ssh_session),
            )
            .route(
                "/wallguard/gateway/tty",
                web::to(tty_gateway::open_tty_session),
            )
            .default_service(web::to(proxy::proxy_http_request))
    })
    .bind(config.addr)
    .unwrap()
    .run()
    .await
    .unwrap()
}
