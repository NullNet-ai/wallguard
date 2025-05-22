use crate::cli::Args;
use crate::constants::DISK_SIZE;
use crate::data_transmission::dump_dir::DumpDir;
use crate::data_transmission::grpc_handler::handle_connection_and_retransmission;
use crate::data_transmission::packets::transmitter::transmit_packets;
use crate::data_transmission::resources::transmitter::monitor_system_resources;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub(crate) async fn spawn_long_running_tasks(args: Args, token: Arc<RwLock<String>>) {
    // for long-running tasks (i.e., packets and system resources transmission),
    // we need to properly handle reconnections: use a separate task to check if the interface is still healthy
    let client = Arc::new(Mutex::new(None));
    let client_2 = client.clone();
    let client_3 = client.clone();

    let dump_bytes = (u64::from(args.disk_percentage) * *DISK_SIZE) / 100;
    log::info!(
        "Will use at most {dump_bytes} bytes of disk space for packets and resources dump files"
    );
    let dump_dir = DumpDir::new(dump_bytes).await;
    let dump_dir_2 = dump_dir.clone();
    let dump_dir_3 = dump_dir.clone();

    let token_2 = token.clone();
    let token_3 = token.clone();

    let addr = args.addr.clone();
    tokio::spawn(async move {
        handle_connection_and_retransmission(&addr, args.port, client, dump_dir, token).await;
    });

    tokio::spawn(async move {
        transmit_packets(args, token_2, dump_dir_2, client_2).await;
    });

    tokio::spawn(async move {
        monitor_system_resources(token_3, dump_dir_3, client_3).await;
    });
}
