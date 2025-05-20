use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) async fn monitor_system_resources(token: Arc<RwLock<String>>) {
    let rx = nullnet_libresmon::poll_system_resources(1000);
    while let Ok(res) = rx.recv().await {
        // create proper gRPC object including timestamp
        // send it to the server
    }
}
