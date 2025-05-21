use crate::cli::Args;
use crate::constants::{BATCH_SIZE, DISK_SIZE, QUEUE_SIZE};
use crate::packet_transmitter::dump_dir::DumpDir;
use crate::packet_transmitter::grpc_handler::handle_connection_and_retransmission;
use crate::packet_transmitter::packet_buffer::PacketBuffer;
use crate::timer::Timer;
use nullnet_libwallguard::{Packet, Packets, WallGuardGrpcInterface};
use std::cmp::min;
use std::sync::Arc;
use async_channel::Sender;
use tokio::sync::{Mutex, RwLock};

pub(crate) async fn transmit_packets(args: Args, token: Arc<RwLock<String>>) {
    let monitor_config = nullnet_traffic_monitor::MonitorConfig {
        addr: args.addr.clone(),
        snaplen: args.snaplen,
    };
    let mut rx = nullnet_traffic_monitor::monitor_devices(&monitor_config);
    // this channel is used for gRPC client-side packet streaming to the server
    let (tonic_tx, tonic_rx) = async_channel::unbounded();
    let tonic_tx_2 = tonic_tx.clone();

    let dump_bytes = (u64::from(args.disk_percentage) * *DISK_SIZE) / 100;

    log::info!("Will use at most {dump_bytes} bytes of disk space for packet dump files");

    let client = Arc::new(Mutex::new(None));
    let client_2 = client.clone();
    let dump_dir = DumpDir::new(dump_bytes).await;
    let dump_dir_2 = dump_dir.clone();
    let token_2 = token.clone();
    tokio::spawn(async move {
        handle_connection_and_retransmission(
            &tonic_tx_2, tonic_rx,
            &args.addr, args.port, client_2, dump_dir_2, token_2)
            .await;
    });

    let mut packet_batch = PacketBuffer::new(BATCH_SIZE);
    let mut packet_queue = PacketBuffer::new(QUEUE_SIZE);
    let mut timer = Timer::new(args.transmit_interval);
    loop {
        if let Ok(packet) = rx.recv() {
            let packet = Packet {
                timestamp: packet.timestamp,
                interface: packet.interface,
                link_type: packet.link_type,
                data: packet.data,
            };
            packet_batch.push(packet);
            if packet_batch.is_full() || timer.is_expired() {
                timer.reset();

                send_packets(
                    &tonic_tx,
                    &client,
                    &mut packet_batch,
                    &mut packet_queue,
                    args.uuid.clone(),
                    token.read().await.clone(),
                )
                .await;
                if packet_queue.is_full() {
                    dump_dir
                        .dump_packets_to_file(packet_queue.take(), args.uuid.clone())
                        .await;
                    if dump_dir.is_full().await {
                        log::warn!("Dump size maximum limit reached. Entering idle mode...",);
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
    tx: &Sender<Packets>,
    interface: &Arc<Mutex<Option<WallGuardGrpcInterface>>>,
    packet_batch: &mut PacketBuffer,
    packet_queue: &mut PacketBuffer,
    uuid: String,
    token: String,
) {
    packet_queue.extend(packet_batch.take());
    if interface.lock().await.is_some() {
        while !packet_queue.is_empty() {
            let range = ..min(packet_queue.len(), BATCH_SIZE);
            let packets = Packets {
                uuid: uuid.clone(),
                packets: packet_queue.get(range),
                token: token.clone(),
            };
            tx.send(packets).await.unwrap();
            packet_queue.drain(range);
        }
    }
}
