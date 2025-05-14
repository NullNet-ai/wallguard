mod common;
mod proxy;
mod remote_access_request;
mod remote_access_terminate;
mod ssh_gateway;

use crate::app_context::AppContext;
use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};
use proxy::proxy;
use remote_access_request::remote_access_request;
use remote_access_terminate::remote_access_terminate;
use std::net::TcpListener;

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 4444;

pub async fn run_http_server(context: AppContext) {
    let app_state = web::Data::new(context);
    let listener =
        TcpListener::bind(format!("{ADDR}:{PORT}")).expect("Failed to bind to HTTP server addr");

    log::info!("HTTP API listening on http://{ADDR}:{PORT}");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
            ])
            .max_age(3600);
        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .route(
                "/v1/api/remote_access",
                web::post().to(remote_access_request),
            )
            .route(
                "/v1/api/remote_access",
                web::delete().to(remote_access_terminate),
            )
            .route("/v1/api/ssh", web::to(ssh_gateway::open_ssh_session))
            .default_service(web::to(proxy))
    })
    .listen(listener)
    .unwrap()
    .run()
    .await
    .unwrap();
}
