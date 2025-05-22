use crate::constants::QUEUE_SIZE_RESOURCES;
use crate::data_transmission::dump_dir::{DumpDir, DumpItem};
use crate::data_transmission::item_buffer::ItemBuffer;
use chrono::Utc;
use nullnet_libwallguard::{SystemResource, SystemResources, WallGuardGrpcInterface};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub(crate) async fn monitor_system_resources(
    token: Arc<RwLock<String>>,
    dump_dir: DumpDir,
    client: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
) {
    let mut resources_queue = ItemBuffer::new(QUEUE_SIZE_RESOURCES);
    let mut rx = nullnet_libresmon::poll_system_resources(1000);
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
        let resources = SystemResources {
            token: token.read().await.clone(),
            resources: vec![resource.clone()],
        };

        // send it to the server
        if let Some(client) = client.lock().await.as_mut() {
            if client.handle_system_resources(resources).await.is_err() {
                log::error!("Failed to send system resources");
            } else {
                continue;
            }
        }

        // if arrived here something went wrong: store the resources in the queue / dump to file
        resources_queue.push(resource);
        if resources_queue.is_full() {
            log::warn!(
                "Queue is full. Dumping {} system resources to file",
                resources_queue.len()
            );
            let dump_item = DumpItem::Resources(SystemResources {
                resources: resources_queue.take(),
                token: String::new(),
            });
            dump_dir.dump_item_to_file(dump_item).await;
            if dump_dir.is_full().await {
                log::warn!("Dump size maximum limit reached. System resources routine entering idle mode...",);
                // stop current resources monitoring and wait for the server to come up again
                rx.close();
                // wait for the server to come up again
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    if client.lock().await.is_some() {
                        break;
                    }
                }
                // restart resources monitoring
                rx = nullnet_libresmon::poll_system_resources(1000);
            }
        }
    }
}
