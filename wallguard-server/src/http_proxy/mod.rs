use crate::app_context::AppContext;
use crate::http_proxy::api::create_filter_rules;
use crate::http_proxy::api::enable_config_monitoring;
use crate::http_proxy::api::enable_telemetry_monitoring;
use crate::http_proxy::api::enable_traffic_monitoring;
use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};
use api::authorize_device;
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
                "/wallguard/api/v1/authorize_device",
                web::post().to(authorize_device),
            )
            .route(
                "/wallguard/api/v1/enable_traffic_monitoring",
                web::post().to(enable_traffic_monitoring),
            )
            .route(
                "/wallguard/api/v1/enable_telemetry_monitoring",
                web::post().to(enable_telemetry_monitoring),
            )
            .route(
                "/wallguard/api/v1/enable_config_monitoring",
                web::post().to(enable_config_monitoring),
            )
            .route(
                "/wallguard/gateway/ssh",
                web::to(ssh_gateway::open_ssh_session),
            )
            .route(
                "/wallguard/gateway/tty",
                web::to(tty_gateway::open_tty_session),
            )
            .route("/wallguard/rule/filter", web::to(create_filter_rules))
            .default_service(web::to(proxy::proxy_http_request))
    })
    .bind(config.addr)
    .unwrap()
    .run()
    .await
    .unwrap()
}
