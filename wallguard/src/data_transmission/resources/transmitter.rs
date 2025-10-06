use crate::constants::QUEUE_SIZE_RESOURCES;
use crate::data_transmission::dump_dir::{DumpDir, DumpItem};
use crate::data_transmission::item_buffer::ItemBuffer;
use crate::token_provider::TokenProvider;
use crate::wg_server::WGServer;
use async_channel::Receiver;
use chrono::Utc;
use wallguard_common::protobuf::wallguard_service::{SystemResource, SystemResourcesData};

pub(crate) async fn transmit_system_resources(
    rx: Receiver<nullnet_libresmon::SystemResources>,
    token_provider: TokenProvider,
    dump_dir: DumpDir,
    client: WGServer,
) {
    let mut resources_queue = ItemBuffer::new(QUEUE_SIZE_RESOURCES);
    while let Ok(res) = rx.recv().await {
        // create proper gRPC object including token and timestamp
        #[allow(clippy::cast_possible_wrap)]
        let resource = SystemResource {
            timestamp: Utc::now().to_rfc3339(),
            num_cpus: res.num_cpus as i64,
            global_cpu_usage: res.global_cpu_usage,
            cpu_usages: format!("{:?}", res.cpu_usages.into_iter().collect::<Vec<_>>()),
            total_memory: res.total_memory as i64,
            used_memory: res.used_memory as i64,
            total_disk_space: res.total_disk_space as i64,
            available_disk_space: res.available_disk_space as i64,
            read_bytes: res.read_bytes as i64,
            written_bytes: res.written_bytes as i64,
            temperatures: format!(
                "{:?}",
                res.temperatures
                    .into_iter()
                    .filter_map(|(k, v)| v.map(|v| (k, v)))
                    .collect::<Vec<_>>()
            ),
        };
        resources_queue.push(resource);

        if let Some(token) = token_provider.get().await {
            let resources = SystemResourcesData {
                token,
                resources: resources_queue.get(..resources_queue.len()),
            };

            // send it to the server

            if client
                .handle_system_resources_data(resources)
                .await
                .is_err()
            {
                log::error!("Failed to send system resources");
            } else {
                resources_queue.drain(..resources_queue.len());
                continue;
            }
        } else {
            log::error!("Faild to obtain a token");
        }

        // if arrived here something went wrong: dump to file if queue is full
        if resources_queue.is_full() {
            log::warn!(
                "Queue is full. Dumping {} system resources to file",
                resources_queue.len()
            );
            let dump_item = DumpItem::Resources(SystemResourcesData {
                resources: resources_queue.take(),
                token: String::new(),
            });
            dump_dir.dump_item_to_file(dump_item).await;
            if dump_dir.is_full().await {
                log::warn!(
                    "Dump size maximum limit reached. System resources routine entering idle mode...",
                );
                // wait for the server to come up again
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    if client.is_connected().await {
                        break;
                    }
                }
            }
        }
    }
}
