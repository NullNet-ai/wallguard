use crate::authentication::AuthHandler;
use crate::constants::BATCH_SIZE;
use crate::logger::Logger;
use crate::packet_transmitter::dump_dir::DumpDir;
use libwallguard::{Authentication, Packets, WallGuardGrpcInterface};
use std::cmp::min;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

pub(crate) async fn handle_connection_and_retransmission(
    addr: &str,
    port: u16,
    interface: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
    dump_dir: DumpDir,
    auth: AuthHandler,
) {
    loop {
        let Ok(token) = auth.obtain_token_safe().await else {
            Logger::log(log::Level::Error, "Authentication failed");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            continue;
        };

        if interface.lock().await.is_some() {
            if interface
                .lock()
                .await
                .as_mut()
                .unwrap()
                .heartbeat(token)
                .await
                .is_err()
            {
                Logger::log(
                    log::Level::Error,
                    "Failed to send heartbeat. Reconnecting...",
                );
                *interface.lock().await = None;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        } else {
            // wait for the server to come up...
            let client = WallGuardGrpcInterface::new(addr, port).await;
            *interface.lock().await = Some(client);
            // send packets accumulated in dump files
            'file_loop: for file in dump_dir.get_files_sorted().await {
                let bytes = fs::read(file.path()).await.unwrap_or_default();
                let mut dump: Packets = bincode::deserialize(&bytes).unwrap_or_default();
                // update auth token of packets retrieved from disk
                dump.auth = Some(Authentication {
                    token: token.clone(),
                });

                while !dump.packets.is_empty() {
                    let range = ..min(dump.packets.len(), BATCH_SIZE);
                    let packets = Packets {
                        packets: dump.packets.get(range).unwrap_or_default().to_vec(),
                        ..dump.clone()
                    };
                    if interface
                        .lock()
                        .await
                        .as_mut()
                        .unwrap()
                        .handle_packets(packets)
                        .await
                        .is_err()
                    {
                        // server is down again, try again later
                        *interface.lock().await = None;
                        Logger::log(
                            log::Level::Error,
                            "Failed to send packet dump. Reconnecting...",
                        );
                        // update dump file with unsent packets
                        dump_dir.update_dump_file(file.path(), dump).await;
                        break 'file_loop;
                    }
                    // remove sent packets from dump
                    dump.packets.drain(range);
                }

                Logger::log(
                    log::Level::Info,
                    format!("Dump file '{:?}' sent successfully", file.file_name()),
                );
                fs::remove_file(file.path())
                    .await
                    .expect("Failed to remove dump file");
            }
        }
    }
}
