use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use wallguard_server::WallGuardGrpcInterface;

pub(crate) async fn handle_connection_and_retransmission(
    addr: &str,
    port: u16,
    interface: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
) {
    loop {
        if interface.lock().await.is_some() {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            if interface
                .lock()
                .await
                .as_mut()
                .unwrap()
                .heartbeat()
                .await
                .is_err()
            {
                println!("Failed to send heartbeat. Reconnecting...");
                *interface.lock().await = None;
            }
        } else {
            // wait for the server to come up...
            let client = WallGuardGrpcInterface::new(addr, port).await;
            *interface.lock().await = Some(client);
            // send packets accumulated in dump files
            fs::create_dir("packet_dumps").await.unwrap_or_default();
            let mut dir = fs::read_dir("packet_dumps")
                .await
                .expect("Failed to read packet dumps directory");
            let mut files = Vec::new();
            while let Ok(Some(file)) = dir.next_entry().await {
                files.push(file);
            }
            files.sort_by_key(fs::DirEntry::file_name);
            for file in files {
                let dump = fs::read(file.path()).await.unwrap_or_default();
                let packets: wallguard_server::Packets =
                    bincode::deserialize(&dump).unwrap_or_default();
                if interface
                    .lock()
                    .await
                    .as_mut()
                    .unwrap()
                    .handle_packets(packets)
                    .await
                    .is_err()
                {
                    // server is down again, keep packet dumps and try again later
                    println!("Failed to send packet dump. Reconnecting...");
                    *interface.lock().await = None;
                    break;
                }
                println!("Dump file '{:?}' sent successfully", file.file_name());
                fs::remove_file(file.path())
                    .await
                    .expect("Failed to remove dump file");
            }
        }
    }
}
