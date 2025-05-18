use crate::app_context::AppContext;
use api::request_session;
use config::HttpProxyConfig;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};

mod api;
mod config;
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
            .route("/v1/api/remote_access", web::post().to(request_session))
        // .route(
        //     "/v1/api/remote_access",
        //     web::post().to(remote_access_request),
        // )
        // .route(
        //     "/v1/api/remote_access",
        //     web::delete().to(remote_access_terminate),
        // )
        // .route("/v1/api/ssh", web::to(ssh_gateway::open_ssh_session))
        // .default_service(web::to(proxy))
    })
    .bind(config.addr)
    .unwrap()
    .run()
    .await
    .unwrap()
}
