mod remote_access_request;
mod remote_access_terminate;

use crate::app_context::AppContext;
use actix_web::{web, App, HttpServer};
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
        App::new()
            .app_data(app_state.clone())
            .route(
                "/v1/api/remote_access",
                web::post().to(remote_access_request),
            )
            .route(
                "/v1/api/remote_access",
                web::get().to(remote_access_terminate),
            )
    })
    .listen(listener)
    .unwrap()
    .run()
    .await
    .unwrap();
}
