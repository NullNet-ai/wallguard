#![allow(clippy::module_name_repetitions)]

use datastore::DatastoreWrapper;

mod datastore;
mod grpc_server;
mod parser;
mod proto;
mod utils;

#[tokio::main]
async fn main() {
    let datastore = if cfg!(feature = "no-datastore") {
        println!("Datastore functionality disabled");
        None
    } else {
        let datastore = DatastoreWrapper::new();
        Some(datastore)
    };

    grpc_server::run_grpc_server(datastore).await;
}
