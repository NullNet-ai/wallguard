#![allow(clippy::module_name_repetitions)]

use app_context::AppContext;
use tokio::signal;

mod app_context;
use crate::grpc_server::{ADDR, PORT};
use crate::utils::{ACCOUNT_ID, ACCOUNT_SECRET};
use clap::Parser;

mod cli;
mod datastore;
mod grpc_server;
mod http_server;
mod parser;
mod proto;
mod tunnel;
mod utils;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let datastore_logger_config = nullnet_liblogging::DatastoreConfig::new(
        ACCOUNT_ID.as_str(),
        ACCOUNT_SECRET.as_str(),
        ADDR,
        PORT,
    );
    let logger_config =
        nullnet_liblogging::LoggerConfig::new(true, false, Some(datastore_logger_config), vec![]);
    nullnet_liblogging::Logger::init(logger_config);

    let app_context = AppContext::new()
        .await
        .expect("Failed to initialize AppContext");

    tokio::select! {
        _ = grpc_server::run_grpc_server(app_context.clone(), args) => {},
        _ = http_server::run_http_server(app_context) => {},
        _ = signal::ctrl_c() => {}
    };
}
