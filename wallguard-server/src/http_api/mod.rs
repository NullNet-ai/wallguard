use crate::app_context::AppContext;

use crate::http_api::api::authorize_device;
use crate::http_api::api::create_alias;
use crate::http_api::api::create_filter_rule;
use crate::http_api::api::create_nat_rule;
use crate::http_api::api::create_ssh_session;
use crate::http_api::api::create_tunnel;
use crate::http_api::api::delete_ssh_session;
use crate::http_api::api::delete_tunnel;
use crate::http_api::api::enable_config_monitoring;
use crate::http_api::api::enable_telemetry_monitoring;
use crate::http_api::api::enable_traffic_monitoring;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};
use config::HttpApiConfig;

mod api;
mod config;
// mod rd_gateway;
pub mod ssh_gateway_v2;
// mod tty_gateway;
pub mod utilities;

pub async fn run_http_api(context: AppContext) {
    let config = HttpApiConfig::from_env();
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
            .route("/wallguard/api/v1/tunnel", web::post().to(create_tunnel))
            .route("/wallguard/api/v1/tunnel", web::delete().to(delete_tunnel))
            .route(
                "/wallguard/api/v1/ssh_session",
                web::post().to(create_ssh_session),
            )
            .route(
                "/wallguard/api/v1/ssh_session",
                web::delete().to(delete_ssh_session),
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
                web::to(ssh_gateway_v2::open_ssh_session),
            )
            // .route(
            //     "/wallguard/gateway/tty",
            //     web::to(tty_gateway::open_tty_session),
            // )
            // .route(
            //     "/wallguard/gateway/rd",
            //     web::to(rd_gateway::open_remote_desktop_session),
            // )
            .route("/wallguard/rule/filter", web::to(create_filter_rule))
            .route("/wallguard/rule/nat", web::to(create_nat_rule))
            .route("/wallguard/alias", web::to(create_alias))
    })
    .bind(config.addr)
    .unwrap()
    .run()
    .await
    .unwrap()
}
