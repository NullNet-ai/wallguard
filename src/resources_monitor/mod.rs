use chrono::Utc;
use nullnet_libwallguard::{SystemResources, WallGuardGrpcInterface};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub(crate) async fn monitor_system_resources(
    token: Arc<RwLock<String>>,
    client: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
) {
    let rx = nullnet_libresmon::poll_system_resources(1000);
    while let Ok(res) = rx.recv().await {
        // create proper gRPC object including token and timestamp
        #[allow(clippy::cast_possible_wrap)]
        let resources = SystemResources {
            token: token.read().await.clone(),
            timestamp: Utc::now().to_rfc3339(),
            num_cpus: res.num_cpus as i64,
            global_cpu_usage: res.global_cpu_usage,
            cpu_usages: res.cpu_usages,
            total_memory: res.total_memory as i64,
            used_memory: res.used_memory as i64,
            total_disk_space: res.total_disk_space as i64,
            available_disk_space: res.available_disk_space as i64,
            read_bytes: res.read_bytes as i64,
            written_bytes: res.written_bytes as i64,
            temperatures: res
                .temperatures
                .into_iter()
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect(),
        };

        // send it to the server
        if let Some(client) = client.lock().await.as_mut() {
            if client.handle_system_resources(resources).await.is_err() {
                log::error!("Failed to send system resources");
            }
        }
    }
}
