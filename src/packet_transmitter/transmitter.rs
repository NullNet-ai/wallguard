use crate::cli::Args;
use crate::constants::{BATCH_SIZE, DISK_SIZE, QUEUE_SIZE};
use crate::packet_transmitter::dump_dir::DumpDir;
use crate::packet_transmitter::grpc_handler::handle_connection_and_retransmission;
use crate::packet_transmitter::packet_buffer::PacketBuffer;
use std::sync::Arc;
use tokio::sync::Mutex;
use libwallguard::{Authentication, Packet, Packets, WallGuardGrpcInterface};

pub(crate) async fn transmit_packets(args: Args, token: String) {
    let monitor_config = nullnet_traffic_monitor::MonitorConfig {
        addr: args.addr.clone(),
        snaplen: args.snaplen,
    };
    let mut rx = nullnet_traffic_monitor::monitor_devices(&monitor_config);

    let dump_bytes = (u64::from(args.disk_percentage) * *DISK_SIZE) / 100;
    println!("Will use at most {dump_bytes} bytes of disk space for packet dump files");

    let client = Arc::new(Mutex::new(None));
    let client_2 = client.clone();
    let dump_dir = DumpDir::new(dump_bytes).await;
    let dump_dir_2 = dump_dir.clone();
    let token_2 = token.clone();
    tokio::spawn(async move {
        handle_connection_and_retransmission(&args.addr, args.port, client_2, dump_dir_2, token_2)
            .await;
    });

    let mut packet_batch = PacketBuffer::new(BATCH_SIZE);
    let mut packet_queue = PacketBuffer::new(QUEUE_SIZE);
    loop {
        if let Ok(packet) = rx.recv() {
            let packet = Packet {
                timestamp: packet.timestamp,
                interface: packet.interface,
                link_type: packet.link_type,
                data: packet.data,
            };
            packet_batch.push(packet);
            if packet_batch.is_full() {
                send_packets(
                    &client,
                    &mut packet_batch,
                    &mut packet_queue,
                    args.uuid.clone(),
                    token.clone(),
                )
                .await;
                if packet_queue.is_full() {
                    dump_packets_to_file(
                        packet_queue.take(),
                        args.uuid.clone(),
                        &dump_dir,
                        token.clone(),
                    )
                    .await;
                    if dump_dir.is_full().await {
                        println!("Dump size maximum limit reached. Entering idle mode...");
                        // stop traffic monitoring
                        drop(rx);
                        // wait for the server to come up again
                        loop {
                            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                            if client.lock().await.is_some() {
                                break;
                            }
                        }
                        // restart traffic monitoring
                        rx = nullnet_traffic_monitor::monitor_devices(&monitor_config);
                    }
                }
            }
        }
    }
}

async fn send_packets(
    interface: &Arc<Mutex<Option<WallGuardGrpcInterface>>>,
    packet_batch: &mut PacketBuffer,
    packet_queue: &mut PacketBuffer,
    uuid: String,
    token: String,
) {
    packet_queue.extend(packet_batch.take());
    if let Some(client) = interface.lock().await.as_mut() {
        let p = Packets {
            uuid,
            packets: packet_queue.get_clone(),
            auth: Some(Authentication { token }),
        };
        if client.handle_packets(p).await.is_ok() {
            packet_queue.clear();
        };
    }
}

async fn dump_packets_to_file(
    packets: Vec<Packet>,
    uuid: String,
    dump_dir: &DumpDir,
    token: String,
) {
    let now = chrono::Utc::now().to_rfc3339();
    let file_path = dump_dir.get_file_path(&now);
    println!(
        "Queue is full. Dumping {} packets to file '{file_path}'",
        packets.len()
    );
    let dump = Packets {
        uuid,
        packets,
        auth: Some(Authentication { token }),
    };
    tokio::fs::write(
        file_path,
        bincode::serialize(&dump).expect("Failed to serialize packets"),
    )
    .await
    .expect("Failed to write dump file");
}
