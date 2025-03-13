#![allow(clippy::module_name_repetitions)]

use app_context::AppContext;
use tokio::signal;

mod app_context;
mod datastore;
mod grpc_server;
mod http_server;
mod parser;
mod proto;
mod tunnel;
mod utils;

#[tokio::main]
async fn main() {
    // disable logging to datastore until we have an account for authenticating server to log
    // let datastore_logger_config =
    //     nullnet_liblogging::DatastoreConfig::new("account_id", "account_secret", ADDR, PORT);
    let logger_config = nullnet_liblogging::LoggerConfig::new(true, true, None, vec![]);
    nullnet_liblogging::Logger::init(logger_config);

    let app_context = AppContext::new()
        .await
        .expect("Failed to initialize AppContext");

    // Spawns a worker and returns
    app_context.launch_tunnel().await;

    tokio::select! {
        _ = grpc_server::run_grpc_server(app_context.clone()) => {},
        _ = http_server::run_http_server(app_context) => {},
        _ = signal::ctrl_c() => {}
    };
}
