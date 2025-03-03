use axum::{routing::get, Router};
use tokio::net::TcpListener;

mod get_addr;

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 4444;

pub async fn run_http_server() {
    let app = Router::new().route("/v1/api/addr", get(get_addr::get_addr));

    let listener = TcpListener::bind(format!("{ADDR}:{PORT}")).await.unwrap();

    log::info!("HTTP API listening on http://{ADDR}:{PORT}");

    axum::serve(listener, app).await.unwrap();
}
