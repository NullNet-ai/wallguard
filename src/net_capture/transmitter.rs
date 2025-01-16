use crate::constants::{BUFFER_SIZE, CA_CERT};
use crate::proto::traffic_monitor::traffic_monitor_client::TrafficMonitorClient;
use crate::proto::traffic_monitor::{Empty, Packet, Packets};
use std::sync::mpsc::Receiver;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::{Request, Response};

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

pub(crate) async fn transmit_packets(rx: &Receiver<Packet>, addr: String, port: u16, uuid: String) {
    let mut client = grpc_client_setup(addr, port).await;
    let mut packet_buffer = PacketBuffer::new();
    let mut failure_buffer = Vec::new();
    loop {
        if let Ok(packet) = rx.recv() {
            packet_buffer.push_packet(packet);
            if packet_buffer.is_full() {
                if let Err(packets) = send_packets(
                    &mut client,
                    &mut packet_buffer,
                    &mut failure_buffer,
                    uuid.clone(),
                )
                .await
                {
                    println!(
                        "Transmission failed; {} packets queued for later...",
                        packets.len()
                    );
                    failure_buffer.extend(packets);
                }
            }
        }
    }
}

async fn grpc_client_setup(addr: String, port: u16) -> TrafficMonitorClient<Channel> {
    let tls = ClientTlsConfig::new().ca_certificate(CA_CERT.to_owned());

    let Ok(channel) = Channel::from_shared(format!("https://{addr}:{port}"))
        .expect("Failed to parse address")
        .tls_config(tls)
        .expect("Failed to configure up TLS")
        .connect()
        .await
    else {
        println!("Failed to connect to the server. Retrying in 1 second...");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Box::pin(grpc_client_setup(addr, port)).await;
    };

    println!("Connected to the server");
    TrafficMonitorClient::new(channel)
}

async fn send_packets(
    client: &mut TrafficMonitorClient<Channel>,
    packet_buffer: &mut PacketBuffer,
    failure_buffer: &mut Vec<Packet>,
    uuid: String,
) -> Result<Response<Empty>, Vec<Packet>> {
    failure_buffer.extend(packet_buffer.take_packets());
    let p = Packets {
        uuid,
        packets: std::mem::take(failure_buffer),
    };

    client
        .handle_packets(Request::new(p.clone())) // TODO: avoid cloning when possible
        .await
        .map_err(|_| p.packets)
}
