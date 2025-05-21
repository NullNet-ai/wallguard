use crate::constants::BATCH_SIZE;
use crate::packet_transmitter::dump_dir::DumpDir;
use nullnet_libwallguard::{Logs, Packets, WallGuardGrpcInterface};
use std::cmp::min;
use std::sync::Arc;
use async_channel::{Receiver, Sender};
use tokio::fs;
use tokio::sync::{Mutex, RwLock};

pub(crate) async fn handle_connection_and_retransmission(
    tx: &Sender<Packets>,
    rx: Receiver<Packets>,
    addr: &str,
    port: u16,
    interface: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
    dump_dir: DumpDir,
    token: Arc<RwLock<String>>,
) {
    loop {
        if interface.lock().await.is_some() {
            // check if the server is still up (sending empty logs)
            if interface
                .lock()
                .await
                .as_mut()
                .unwrap()
                .handle_logs(Logs {
                    logs: vec![],
                    token: token.read().await.clone(),
                })
                .await
                .is_err()
            {
                log::error!("Failed to send heartbeat. Reconnecting...",);
                *interface.lock().await = None;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        } else {
            // wait for the server to come up...
            let client = WallGuardGrpcInterface::new(addr, port).await;
            *interface.lock().await = Some(client);
            // setup packet gRPC stream
            interface
                .lock()
                .await
                .as_mut()
                .unwrap()
                .handle_packets(rx.clone())
                .await
                .unwrap();

            // send packets accumulated in dump files
            for file in dump_dir.get_files_sorted().await {
                let bytes = fs::read(file.path()).await.unwrap_or_default();
                let mut dump: Packets = bincode::deserialize(&bytes).unwrap_or_default();
                // update auth token of packets retrieved from disk
                dump.token = token.read().await.to_string();

                while !dump.packets.is_empty() {
                    let range = ..min(dump.packets.len(), BATCH_SIZE);
                    let packets = Packets {
                        packets: dump.packets.get(range).unwrap_or_default().to_vec(),
                        ..dump.clone()
                    };
                    tx.send(packets).await.unwrap();
                    // remove sent packets from dump
                    dump.packets.drain(range);
                }

                log::info!("Dump file '{:?}' sent successfully", file.file_name());
                fs::remove_file(file.path())
                    .await
                    .expect("Failed to remove dump file");
            }
        }
    }
}
