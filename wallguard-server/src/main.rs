#![allow(clippy::module_name_repetitions)]

use datastore::DatastoreWrapper;

mod datastore;
mod grpc_server;
mod http_server;
mod parser;
mod proto;
mod utils;

#[tokio::main]
async fn main() {
    nullnet_liblogging::Logger::init(None, "wallgaurd-server", vec![]);

    let datastore = DatastoreWrapper::new()
        .await
        .expect("Failed to connect to the datastore");

    tokio::join!(
        grpc_server::run_grpc_server(datastore),
        http_server::run_http_server()
    );
}
