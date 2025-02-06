use crate::authentication::AuthHandler;
use crate::packet_transmitter::dump_dir::DumpDir;
use libwallguard::WallGuardGrpcInterface;
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
        if interface.lock().await.is_some() {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let Ok(token) = auth.obtain_token_safe().await else {
                eprintln!("Authentication failed");
                continue;
            };

            if interface
                .lock()
                .await
                .as_mut()
                .unwrap()
                .heartbeat(token)
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
            for file in dump_dir.get_files_sorted().await {
                let dump = fs::read(file.path()).await.unwrap_or_default();
                let packets: libwallguard::Packets =
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
