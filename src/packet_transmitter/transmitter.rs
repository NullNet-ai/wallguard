use crate::constants::BUFFER_SIZE;
use std::sync::mpsc::Receiver;
use traffic_monitor::PacketInfo;
use wallguard_server::{Authentication, Packet, Packets, WallGuardGrpcInterface};

struct PacketBuffer {
    buffer: Vec<Packet>,
}

impl PacketBuffer {
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(BUFFER_SIZE),
        }
    }

    fn push_packet(&mut self, packet: Packet) {
        self.buffer.push(packet);
    }

    fn take_packets(&mut self) -> Vec<Packet> {
        std::mem::take(&mut self.buffer)
    }

    fn is_full(&self) -> bool {
        self.buffer.len() >= BUFFER_SIZE
    }
}

pub(crate) async fn transmit_packets(
    rx: &Receiver<PacketInfo>,
    addr: String,
    port: u16,
    uuid: String,
    token: String,
) {
    let mut client = WallGuardGrpcInterface::new(&addr, port).await;
    let mut packet_buffer = PacketBuffer::new();
    let mut failure_buffer = Vec::new();
    loop {
        if let Ok(packet) = rx.recv() {
            let packet = Packet {
                timestamp: packet.timestamp,
                interface: packet.interface,
                link_type: packet.link_type,
                data: packet.data,
            };
            packet_buffer.push_packet(packet);
            if packet_buffer.is_full() {
                if let Err(packets) = send_packets(
                    &mut client,
                    &mut packet_buffer,
                    &mut failure_buffer,
                    uuid.clone(),
                    token.clone(),
                )
                .await
                {
                    println!("{} packets queued for later...", packets.len());
                    failure_buffer.extend(packets);
                }
            }
        }
    }
}

async fn send_packets(
    client: &mut WallGuardGrpcInterface,
    packet_buffer: &mut PacketBuffer,
    failure_buffer: &mut Vec<Packet>,
    uuid: String,
    token: String,
) -> Result<(), Vec<Packet>> {
    failure_buffer.extend(packet_buffer.take_packets());
    let p = Packets {
        uuid,
        packets: std::mem::take(failure_buffer),
        auth: Some(Authentication { token }),
    };

    client
        .handle_packets(p.clone()) // TODO: avoid cloning when possible
        .await
        .map_err(|e| {
            println!("Failed to send packets to the server: {e}");
            p.packets
        })
}
