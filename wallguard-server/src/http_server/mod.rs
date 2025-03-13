mod remote_access_request;
mod state;

use crate::tunnel::TunnelServer;
use actix_web::{web, App, HttpServer};
use remote_access_request::remote_access_request;
use state::State;
use std::{net::TcpListener, sync::Arc};
use tokio::sync::Mutex;

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 4444;

pub async fn run_http_server(tunnel: Arc<Mutex<TunnelServer>>) {
    let app_state = web::Data::new(State { tunnel });
    let listener =
        TcpListener::bind(format!("{ADDR}:{PORT}")).expect("Failed to bind to HTTP server addr");

    log::info!("HTTP API listening on http://{ADDR}:{PORT}");

    HttpServer::new(move || {
        App::new().app_data(app_state.clone()).route(
            "/v1/api/remote_access",
            web::post().to(remote_access_request),
        )
    })
    .listen(listener)
    .unwrap()
    .run()
    .await
    .unwrap();
}
