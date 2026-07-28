use crate::constants::BATCH_SIZE;
use crate::data_transmission::dump_dir::{DumpDir, DumpItem};
use crate::token_provider::TokenProvider;
use crate::wg_server::WGServer;
use std::cmp::min;
use std::time::Duration;
use tokio::fs;
use wallguard_common::protobuf::wallguard_service::{ConnectionsData, SystemResourcesData};

pub(crate) async fn handle_connection_and_retransmission(
    interface: WGServer,
    dump_dir: DumpDir,
    token_provider: TokenProvider,
) {
    loop {
        loop {
            if interface.is_connected().await {
                break;
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }

        while token_provider.get().await.is_none() {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let token = token_provider.get().await.unwrap();

        let files = dump_dir.get_files_sorted().await;
        if files.is_empty() {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        // send packets accumulated in dump files
        'file_loop: for file in files {
            let Ok(string) = fs::read_to_string(file.path()).await else {
                continue;
            };
            // Deserializing a dump file (up to the full queue, e.g. 1M
            // records) is CPU-bound; keep it off the tokio runtime so it
            // can't stall gRPC/heartbeat traffic sharing the same executor.
            let Ok(Ok(mut dump)) =
                tokio::task::spawn_blocking(move || serde_json::from_str::<DumpItem>(&string))
                    .await
            else {
                continue;
            };
            // update auth token of items retrieved from disk
            dump.set_token(token.clone());

            // Batches are sliced by a `sent` offset rather than drained from
            // the front on every iteration: draining a Vec's front repeatedly
            // shifts the remaining tail down each time (O(remaining) per
            // batch), which turns replaying a large backlog file into an
            // O(n^2) sequence of memmoves. Slicing leaves the vector
            // untouched until a single drain(..sent) at the end.
            let total = dump.size();
            let mut sent = 0;
            let mut failed = false;

            while sent < total {
                let range = ..min(total - sent, BATCH_SIZE);
                let send_res = match &dump {
                    DumpItem::Connections(c) => {
                        let msg = ConnectionsData {
                            token: c.token.clone(),
                            connections: c.connections[sent..][range].to_vec(),
                        };
                        interface.handle_connections_data(msg).await
                    }
                    DumpItem::Resources(r) => {
                        let msg = SystemResourcesData {
                            token: r.token.clone(),
                            resources: r.resources[sent..][range].to_vec(),
                        };
                        interface.handle_system_resources_data(msg).await
                    }
                    DumpItem::Empty => {
                        log::warn!("Invalid dump file found. Skipping...");
                        continue 'file_loop;
                    }
                };
                if send_res.is_err() {
                    // Server rejected the send even though `is_connected()`
                    // still reports true (e.g. an invalid/expired token) —
                    // back off before retrying instead of immediately
                    // re-reading and re-sending the same file in a tight loop.
                    log::warn!("Failed to send dump. Reconnecting...",);
                    failed = true;
                    break;
                }
                sent += range.end;
            }

            if failed {
                // remove the items that did get sent, in one shot, and persist the rest
                dump.drain(..sent);
                dump_dir.update_items_dump_file(file.path(), dump).await;
                tokio::time::sleep(Duration::from_secs(10)).await;
                break 'file_loop;
            }

            log::info!("Dump file '{:?}' sent successfully", file.file_name());
            fs::remove_file(file.path())
                .await
                .expect("Failed to remove dump file");
        }
    }
}
