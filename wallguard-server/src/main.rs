#![allow(clippy::module_name_repetitions)]

use datastore::DatastoreWrapper;
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
        let datastore = DatastoreWrapper::new();
        Some(datastore)
    };

    for _ in 1..=workers_amount {
        let receiver = rx.clone();
        let ds_client = datastore.clone();

        workers.push(tokio::spawn(async move {
            message_handler::worker_task(receiver, ds_client).await;
        }));
    }

    grpc_server::run_grpc_server(tx, datastore).await;

    for worker in workers {
        worker.await.unwrap();
    }
}
