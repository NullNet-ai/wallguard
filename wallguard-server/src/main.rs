#![allow(clippy::module_name_repetitions)]

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

    // disable logging to datastore until we have an account for authenticating server to log
    // let datastore_logger_config =
    //     nullnet_liblogging::DatastoreConfig::new("account_id", "account_secret", ADDR, PORT);
    let logger_config = nullnet_liblogging::LoggerConfig::new(true, false, None, vec![]);
    nullnet_liblogging::Logger::init(logger_config);

    tokio::join!(
        grpc_server::run_grpc_server(args),
        http_server::run_http_server()
    );
}
