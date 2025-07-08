use crate::constants::{BATCH_SIZE, DATA_TRANSMISSION_INTERVAL_SECONDS, QUEUE_SIZE};
use crate::data_transmission::dump_dir::{DumpDir, DumpItem};
use crate::data_transmission::item_buffer::ItemBuffer;
use crate::timer::Timer;
use crate::token_provider::TokenProvider;
use crate::wg_server::WGServer;
use async_channel::Receiver;
use nullnet_libwallguard::{Packet, PacketsData};
use nullnet_traffic_monitor::PacketInfo;
use std::cmp::min;

pub(crate) async fn transmit_packets(
    rx: Receiver<PacketInfo>,
    token_provider: TokenProvider,
    dump_dir: DumpDir,
    client: WGServer,
) {
    let mut packet_batch = ItemBuffer::new(BATCH_SIZE);
    let mut packet_queue = ItemBuffer::new(QUEUE_SIZE);
    let mut timer = Timer::new(DATA_TRANSMISSION_INTERVAL_SECONDS);

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
                &token_provider,
            )
            .await;

            if packet_queue.is_full() {
                log::warn!(
                    "Queue is full. Dumping {} packets to file",
                    packet_queue.len(),
                );
                let dump_item = DumpItem::Packets(PacketsData {
                    packets: packet_queue.take(),
                    token: String::new(),
                });
                dump_dir.dump_item_to_file(dump_item).await;
                if dump_dir.is_full().await {
                    log::warn!(
                        "Dump size maximum limit reached. Packets routine entering idle mode...",
                    );

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
}

async fn send_packets(
    interface: &WGServer,
    packet_batch: &mut ItemBuffer<Packet>,
    packet_queue: &mut ItemBuffer<Packet>,
    token_provider: &TokenProvider,
) {
    packet_queue.extend(packet_batch.take());
    while !packet_queue.is_empty() {
        let token = token_provider.get().await;

        if token.is_none() {
            log::error!("Failed to obtain token");
            break;
        }

        let range = ..min(packet_queue.len(), BATCH_SIZE);
        let packets = PacketsData {
            packets: packet_queue.get(range),
            token: token.unwrap(),
        };

        if interface.handle_packets_data(packets).await.is_err() {
            log::error!("Failed to send packets");
            break;
        }

        packet_queue.drain(range);
    }
}
