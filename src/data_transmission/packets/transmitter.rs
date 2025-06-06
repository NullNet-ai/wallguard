use crate::cli::Args;
use crate::constants::{BATCH_SIZE, QUEUE_SIZE};
use crate::data_transmission::dump_dir::{DumpDir, DumpItem};
use crate::data_transmission::item_buffer::ItemBuffer;
use crate::timer::Timer;
use async_channel::Receiver;
use nullnet_libwallguard::{Packet, Packets, WallGuardGrpcInterface};
use nullnet_traffic_monitor::PacketInfo;
use std::cmp::min;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub(crate) async fn transmit_packets(
    rx: Receiver<PacketInfo>,
    args: Args,
    token: Arc<RwLock<String>>,
    dump_dir: DumpDir,
    client: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
) {
    let mut packet_batch = ItemBuffer::new(BATCH_SIZE);
    let mut packet_queue = ItemBuffer::new(QUEUE_SIZE);
    let mut timer = Timer::new(args.transmit_interval);
    while let Ok(packet) = rx.recv().await {
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
                &client,
                &mut packet_batch,
                &mut packet_queue,
                token.read().await.clone(),
            )
            .await;
            if packet_queue.is_full() {
                log::warn!(
                    "Queue is full. Dumping {} packets to file",
                    packet_queue.len(),
                );
                let dump_item = DumpItem::Packets(Packets {
                    packets: packet_queue.take(),
                    token: String::new(),
                });
                dump_dir.dump_item_to_file(dump_item).await;
                if dump_dir.is_full().await {
                    log::warn!(
                        "Dump size maximum limit reached. Packets routine entering idle mode...",
                    );
                    // wait for the server to come up again
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        if client.lock().await.is_some() {
                            break;
                        }
                    }
                }
            }
        }
    }
}

async fn send_packets(
    interface: &Arc<Mutex<Option<WallGuardGrpcInterface>>>,
    packet_batch: &mut ItemBuffer<Packet>,
    packet_queue: &mut ItemBuffer<Packet>,
    token: String,
) {
    packet_queue.extend(packet_batch.take());
    if let Some(client) = interface.lock().await.as_mut() {
        while !packet_queue.is_empty() {
            let range = ..min(packet_queue.len(), BATCH_SIZE);
            let packets = Packets {
                packets: packet_queue.get(range),
                token: token.clone(),
            };
            if client.handle_packets(packets).await.is_err() {
                log::error!("Failed to send packets");
                break;
            }
            packet_queue.drain(range);
        }
    }
}
