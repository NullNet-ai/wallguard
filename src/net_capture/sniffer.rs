use crate::cli::Args;
use crate::net_capture::transmitter::transmit_packets;
use crate::proto::traffic_monitor::Packet;
use chrono::Utc;
use pcap::Device;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

pub(crate) async fn sniff_devices(args: Args) {
    let (tx, rx) = mpsc::channel();

    let bpf_program = bpf_program(&args.addr, args.port);
    for device in Device::list().into_iter().flatten() {
        let tx = tx.clone();
        let bpf_program = bpf_program.clone();
        thread::spawn(move || {
            sniff_device(device, &tx, args.snaplen, &bpf_program);
        });
    }

    transmit_packets(&rx, args.addr, args.port, args.uuid).await;
}

fn sniff_device(device: Device, tx: &Sender<Packet>, snaplen: i32, bpf_program: &str) {
    let device_name = device.name.clone();

    let Ok(mut cap) = pcap::Capture::from_device(device)
        .expect("capture initialization error")
        .promisc(true)
        .snaplen(snaplen) // limit stored packets slice dimension (to keep more in the buffer)
        .immediate_mode(true) // parse packets ASAP!
        .open()
    else {
        return;
    };

    cap.filter(bpf_program, true).expect("PCAP filter error");

    let link_type = cap.get_datalink().0;

    loop {
        if let Ok(p) = cap.next_packet() {
            let packet = Packet {
                interface: device_name.clone(),
                data: p.data[..].to_vec(),
                link_type,
                timestamp: Utc::now().to_rfc3339(),
            };
            tx.send(packet).expect("send error");
        }
    }
}

fn bpf_program(addr: &str, port: u16) -> String {
    let ip_addr = format!("{addr}:{port}")
        .to_socket_addrs()
        .expect("Failed to resolve address")
        .next()
        .expect("Failed to get address")
        .ip()
        .to_string();

    let bpf_program = format!("host not {ip_addr}");
    println!("BPF Program: {bpf_program}");
    bpf_program
}
