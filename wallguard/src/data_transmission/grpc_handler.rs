use crate::constants::BATCH_SIZE;
use crate::data_transmission::dump_dir::{DumpDir, DumpItem};
use crate::token_provider::TokenProvider;
use crate::wg_server::WGServer;
use std::cmp::min;
use std::time::Duration;
use tokio::fs;
use wallguard_common::protobuf::wallguard_service::{PacketsData, SystemResourcesData};

pub(crate) async fn _handle_connection_and_retransmission(
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

        // send packets accumulated in dump files
        'file_loop: for file in dump_dir._get_files_sorted().await {
            let Ok(string) = fs::read_to_string(file.path()).await else {
                continue;
            };
            let Ok(mut dump) = serde_json::from_str::<DumpItem>(&string) else {
                continue;
            };
            // update auth token of items retrieved from disk
            dump._set_token(token.clone());

            while dump._size() != 0 {
                let range = ..min(dump._size(), BATCH_SIZE);
                let send_res = match &dump {
                    DumpItem::Packets(p) => {
                        let msg = PacketsData {
                            packets: p.packets.get(range).unwrap_or_default().to_vec(),
                            ..p.clone()
                        };
                        interface.handle_packets_data(msg).await
                    }
                    DumpItem::Resources(r) => {
                        let msg = SystemResourcesData {
                            resources: r.resources.get(range).unwrap_or_default().to_vec(),
                            ..r.clone()
                        };
                        interface.handle_system_resources_data(msg).await
                    }
                    DumpItem::Empty => {
                        log::warn!("Invalid dump file found. Skipping...");
                        continue 'file_loop;
                    }
                };
                if send_res.is_err() {
                    // server is down again, try again later
                    log::warn!("Failed to send dump. Reconnecting...",);
                    // update dump file with unsent items
                    dump_dir._update_items_dump_file(file.path(), dump).await;
                    break 'file_loop;
                }
                // remove sent items from dump
                dump._drain(range);
            }

            log::info!("Dump file '{:?}' sent successfully", file.file_name());
            fs::remove_file(file.path())
                .await
                .expect("Failed to remove dump file");
        }
    }
}
