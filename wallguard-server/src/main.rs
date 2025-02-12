#![allow(clippy::module_name_repetitions)]

use datastore::DatastoreWrapper;

mod datastore;
mod grpc_server;
mod parser;
mod proto;
mod utils;

#[tokio::main]
async fn main() {
    let datastore = DatastoreWrapper::new();
    grpc_server::run_grpc_server(datastore).await;
}
