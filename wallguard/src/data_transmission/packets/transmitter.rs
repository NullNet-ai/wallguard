use super::parser::parse_packets;
use crate::constants::{DATA_TRANSMISSION_INTERVAL_SECONDS, QUEUE_SIZE};
use crate::data_transmission::dump_dir::{DumpDir, DumpItem};
use crate::data_transmission::item_buffer::ItemBuffer;
use crate::timer::Timer;
use crate::token_provider::TokenProvider;
use crate::wg_server::WGServer;
use async_channel::Receiver;
use nullnet_traffic_monitor::PacketInfo;
use std::cmp::min;
use wallguard_common::protobuf::wallguard_service::{Connection, ConnectionsData};

pub(crate) async fn transmit_packets(
    rx: Receiver<PacketInfo>,
    token_provider: TokenProvider,
    dump_dir: DumpDir,
    client: WGServer,
    batch_size: usize,
) {
    // PacketInfo doesn't implement Clone so we use a plain Vec for the raw accumulation window
    let mut raw_batch: Vec<PacketInfo> = Vec::with_capacity(batch_size);
    let mut connection_queue: ItemBuffer<Connection> = ItemBuffer::new(QUEUE_SIZE);
    let mut timer = Timer::new(DATA_TRANSMISSION_INTERVAL_SECONDS);

    while let Ok(packet) = rx.recv().await {
        raw_batch.push(packet);
        if raw_batch.len() >= batch_size || timer.is_expired() {
            timer.reset();

            let connections = parse_packets(std::mem::take(&mut raw_batch));
            connection_queue.extend(connections);

            send_connections(&client, &mut connection_queue, &token_provider, batch_size).await;

            if connection_queue.is_full() {
                log::warn!(
                    "Queue is full. Dumping {} connections to file",
                    connection_queue.len(),
                );
                let dump_item = DumpItem::Connections(ConnectionsData {
                    connections: connection_queue.take(),
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

async fn send_connections(
    interface: &WGServer,
    connection_queue: &mut ItemBuffer<Connection>,
    token_provider: &TokenProvider,
    batch_size: usize,
) {
    while !connection_queue.is_empty() {
        let token = token_provider.get().await;

        if token.is_none() {
            log::error!("Failed to obtain token");
            break;
        }

        let range = ..min(connection_queue.len(), batch_size);
        let data = ConnectionsData {
            connections: connection_queue.get(range),
            token: token.unwrap(),
        };

        if let Err(e) = interface.handle_connections_data(data).await {
            log::error!("Failed to send connections: {e:?}");
            break;
        }

        connection_queue.drain(range);
    }
}
