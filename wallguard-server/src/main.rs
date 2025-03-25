#![allow(clippy::module_name_repetitions)]

use crate::grpc_server::{ADDR, PORT};
use crate::utils::{ACCOUNT_ID, ACCOUNT_SECRET};
use clap::Parser;

mod cli;
mod datastore;
mod grpc_server;
mod http_server;
mod parser;
mod proto;
mod utils;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    let datastore_logger_config =
        nullnet_liblogging::DatastoreConfig::new(ACCOUNT_ID, ACCOUNT_SECRET, ADDR, PORT);
    let logger_config =
        nullnet_liblogging::LoggerConfig::new(true, false, Some(datastore_logger_config), vec![]);
    nullnet_liblogging::Logger::init(logger_config);

    tokio::join!(
        grpc_server::run_grpc_server(args),
        http_server::run_http_server()
    );
}
