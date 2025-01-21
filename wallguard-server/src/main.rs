#![allow(clippy::module_name_repetitions)]

use datastore::{client::DatastoreClient, config::DatastoreConfig};
use proto::wallguard::Packets;

mod datastore;
mod grpc_server;
mod message_handler;
mod parser;
mod proto;

#[tokio::main]
async fn main() {
    let (tx, rx) = async_channel::unbounded::<Packets>();

    let workers_amount: u32 = 64;
    let mut workers = Vec::new();

    let datastore = if cfg!(feature = "no-datastore") {
        println!("Datastore functionality disabled");
        None
    } else {
        let datastore_config = DatastoreConfig::from_env();
        let mut datastore = DatastoreClient::connect(datastore_config)
            .await
            .expect("Could not connect to the datastore");

        datastore
            .handle_authentication()
            .await
            .expect("Datastore Authentication failed");

        Some(datastore)
    };

    for _ in 1..=workers_amount {
        let receiver = rx.clone();
        let ds_client = datastore.clone();

        workers.push(tokio::spawn(async move {
            message_handler::worker_task(receiver, ds_client).await;
        }));
    }

    grpc_server::run_grpc_server(tx).await;

    for worker in workers {
        worker.await.unwrap();
    }
}
