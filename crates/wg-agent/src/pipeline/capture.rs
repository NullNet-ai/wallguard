use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::config::ServerConfig;
use crate::proto::data::{Direction, Packet};

// ---------------------------------------------------------------------------
// BPF exclusion filter for management traffic
// ---------------------------------------------------------------------------

/// Build a BPF fragment that excludes WallGuard management traffic so it never
/// appears in the captured telemetry or the traffic chart.
///
/// With resolved IPs we can be precise: only traffic to/from the server on the
/// management ports is excluded.  If DNS resolution failed at startup we fall
/// back to a port-only filter (less precise but avoids a feedback loop).
fn build_exclusion_filter(server: &ServerConfig, server_ips: &[IpAddr]) -> String {
    let ports = format!(
        "port {} or port {} or port {}",
        server.grpc_port, server.quic_port, server.tcp_port,
    );

    if server_ips.is_empty() {
        // DNS failed — exclude by port only.
        warn!(
            "capture: could not resolve server hostname '{}'; \
             excluding management ports {} {} {} regardless of destination",
            server.name, server.grpc_port, server.quic_port, server.tcp_port,
        );
        format!("not ({ports})")
    } else {
        let hosts = server_ips
            .iter()
            .map(|ip| format!("host {ip}"))
            .collect::<Vec<_>>()
            .join(" or ");
        format!("not (({hosts}) and ({ports}))")
    }
}

/// Spawn one blocking capture task per non-loopback, running network interface.
///
/// Requires `CAP_NET_RAW` (Linux) or equivalent privilege.  When no suitable
/// interface is found the function returns without spawning — the pipeline
/// channel stays open via the sender held in `main.rs`.
pub fn spawn(tx: mpsc::Sender<Packet>, server: &ServerConfig, server_ips: Vec<IpAddr>) {
    let exclusion = build_exclusion_filter(server, &server_ips);
    metrics::gauge!("wg_agent_capture_queue_depth").set(0.0);

    let devices = match pcap::Device::list() {
        Ok(list) => list
            .into_iter()
            .filter(|d| {
                !d.flags.is_loopback()
                    && d.flags.is_up()
                    && d.flags.is_running()
                    && !d.addresses.is_empty()
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            warn!("capture: cannot list interfaces — {e}; packet capture disabled");
            return;
        }
    };

    if devices.is_empty() {
        info!("capture: no suitable interfaces found; packet capture inactive");
        return;
    }

    // Union of all local IPs for inbound/outbound classification.
    let local_ips: HashSet<IpAddr> = devices
        .iter()
        .flat_map(|d| d.addresses.iter().map(|a| a.addr))
        .collect();

    info!(
        interfaces = ?devices.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
        "capture: starting"
    );

    let bpf = format!("(ip or ip6) and {exclusion}");

    for dev in devices {
        let tx_clone  = tx.clone();
        let ips_clone = local_ips.clone();
        let bpf_clone = bpf.clone();
        tokio::task::spawn_blocking(move || capture_loop(dev, tx_clone, ips_clone, bpf_clone));
    }
}

// ---------------------------------------------------------------------------
// Per-interface blocking capture loop
// ---------------------------------------------------------------------------

fn capture_loop(dev: pcap::Device, tx: mpsc::Sender<Packet>, local_ips: HashSet<IpAddr>, bpf: String) {
    let name = dev.name.clone();

    let mut cap = match pcap::Capture::from_device(dev)
        .and_then(|b| b.promisc(false).snaplen(96).timeout(500).open())
    {
        Ok(c) => c,
        Err(e) => {
            warn!(%name, "capture: open failed — {e}");
            return;
        }
    };

    if let Err(e) = cap.filter(&bpf, true) {
        // BPF failed (old kernel, virtual NIC, etc.) — log and continue without
        // the filter rather than disabling capture entirely.  Management traffic
        // will be captured but that is preferable to losing all telemetry.
        warn!(%name, %bpf, "capture: BPF filter not applied — {e}");
    } else {
        debug!(%name, %bpf, "capture: BPF filter applied");
    }

    info!(%name, "capture: active");

    loop {
        match cap.next_packet() {
            Err(pcap::Error::TimeoutExpired) => {
                if tx.is_closed() { break; }
                continue;
            }
            Err(pcap::Error::NoMorePackets) => break,
            Err(e) => {
                warn!(%name, "capture: read error — {e}");
                break;
            }
            Ok(raw) => {
                let ts_ms = raw.header.ts.tv_sec as u64 * 1_000
                    + raw.header.ts.tv_usec as u64 / 1_000;
                let wire_len = raw.header.len;

                if let Some(pkt) = parse_ethernet(raw.data, ts_ms, wire_len, &local_ips) {
                    let depth = tx.max_capacity().saturating_sub(tx.capacity()) as f64;
                    metrics::gauge!("wg_agent_capture_queue_depth").set(depth);

                    match tx.try_send(pkt) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            metrics::counter!("wg_agent.packets.dropped").increment(1);
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => break,
                    }
                }
            }
        }
    }

    info!(%name, "capture: stopped");
}

// ---------------------------------------------------------------------------
// Packet parsing (Ethernet II → IPv4/IPv6 → TCP/UDP)
// ---------------------------------------------------------------------------

fn parse_ethernet(
    data:      &[u8],
    ts_ms:     u64,
    bytes:     u32,
    local_ips: &HashSet<IpAddr>,
) -> Option<Packet> {
    // Ethernet II: dst(6) + src(6) + ethertype(2) = 14 bytes minimum.
    if data.len() < 14 {
        return None;
    }
    let ethertype = u16::from_be_bytes([data[12], data[13]]);
    match ethertype {
        0x0800 => parse_ipv4(&data[14..], ts_ms, bytes, local_ips),
        0x86DD => parse_ipv6(&data[14..], ts_ms, bytes, local_ips),
        0x8100 => {
            // 802.1Q VLAN tag: TCI(2) + inner ethertype(2)
            if data.len() < 18 {
                return None;
            }
            match u16::from_be_bytes([data[16], data[17]]) {
                0x0800 => parse_ipv4(&data[18..], ts_ms, bytes, local_ips),
                0x86DD => parse_ipv6(&data[18..], ts_ms, bytes, local_ips),
                _ => None,
            }
        }
        _ => None,
    }
}

fn parse_ipv4(
    data:      &[u8],
    ts_ms:     u64,
    bytes:     u32,
    local_ips: &HashSet<IpAddr>,
) -> Option<Packet> {
    if data.len() < 20 {
        return None;
    }
    let ihl = ((data[0] & 0x0f) as usize) * 4;
    if ihl < 20 || data.len() < ihl {
        return None;
    }
    let proto  = data[9];
    let src_ip = IpAddr::V4(Ipv4Addr::new(data[12], data[13], data[14], data[15]));
    let dst_ip = IpAddr::V4(Ipv4Addr::new(data[16], data[17], data[18], data[19]));
    let (src_port, dst_port) = extract_ports(proto, data.get(ihl..)?);
    Some(Packet {
        timestamp_ms: ts_ms,
        src_ip:       src_ip.to_string(),
        dst_ip:       dst_ip.to_string(),
        src_port,
        dst_port,
        protocol:  proto as u32,
        bytes,
        direction: classify(src_ip, dst_ip, local_ips),
    })
}

fn parse_ipv6(
    data:      &[u8],
    ts_ms:     u64,
    bytes:     u32,
    local_ips: &HashSet<IpAddr>,
) -> Option<Packet> {
    // Fixed IPv6 header is 40 bytes.
    if data.len() < 40 {
        return None;
    }
    let next_header = data[6];
    let src_ip = IpAddr::V6(Ipv6Addr::from(
        <[u8; 16]>::try_from(&data[8..24]).ok()?,
    ));
    let dst_ip = IpAddr::V6(Ipv6Addr::from(
        <[u8; 16]>::try_from(&data[24..40]).ok()?,
    ));
    let (src_port, dst_port) = extract_ports(next_header, data.get(40..)?);
    Some(Packet {
        timestamp_ms: ts_ms,
        src_ip:       src_ip.to_string(),
        dst_ip:       dst_ip.to_string(),
        src_port,
        dst_port,
        protocol:  next_header as u32,
        bytes,
        direction: classify(src_ip, dst_ip, local_ips),
    })
}

/// Return (src_port, dst_port) from the first 4 bytes of a TCP or UDP header.
/// Returns (0, 0) for all other protocols.
fn extract_ports(proto: u8, data: &[u8]) -> (u32, u32) {
    if data.len() < 4 {
        return (0, 0);
    }
    match proto {
        6 | 17 => (
            u16::from_be_bytes([data[0], data[1]]) as u32,
            u16::from_be_bytes([data[2], data[3]]) as u32,
        ),
        _ => (0, 0),
    }
}

fn classify(src: IpAddr, dst: IpAddr, local_ips: &HashSet<IpAddr>) -> i32 {
    if local_ips.contains(&dst) {
        Direction::In as i32
    } else if local_ips.contains(&src) {
        Direction::Out as i32
    } else {
        Direction::Unknown as i32
    }
}
