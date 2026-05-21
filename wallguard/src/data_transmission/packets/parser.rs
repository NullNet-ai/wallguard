use etherparse::err::ip::{HeaderError, LaxHeaderSliceError};
use etherparse::err::{Layer, LenError};
use etherparse::{LaxPacketHeaders, LenSource, LinkHeader, NetHeaders, TransportHeader};
use nullnet_liberror::{ErrorHandler, Location, location};
use nullnet_traffic_monitor::PacketInfo;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use wallguard_common::protobuf::wallguard_service::Connection;

#[derive(Debug, Hash, Eq, PartialEq)]
struct ConnectionKey {
    interface: String,
    source_ip: IpAddr,
    destination_ip: IpAddr,
    source_port: Option<u16>,
    destination_port: Option<u16>,
    protocol: &'static str,
}

struct ConnectionValue {
    timestamp: String,
    total_packet: u32,
    total_byte: u64,
}

pub fn parse_packets(packets: Vec<PacketInfo>) -> Vec<Connection> {
    let mut map: HashMap<ConnectionKey, ConnectionValue> = HashMap::new();

    for packet in packets {
        if is_ignored_interface(&packet.interface) {
            continue;
        }

        let link_type = packet.link_type;
        let Some(headers) = get_packet_headers(&packet.data, link_type) else {
            continue;
        };
        let Some((source_ip, destination_ip, packet_length)) = extract_ip(&headers.net) else {
            continue;
        };

        let Some((source_port, destination_port, protocol)) =
            extract_transport(&headers.transport)
        else {
            continue;
        };

        let has_eth = matches!(headers.link, Some(LinkHeader::Ethernet2(_)));
        let total_byte = (14 * usize::from(has_eth) + usize::from(packet_length)) as u64;

        let key = ConnectionKey {
            interface: packet.interface,
            source_ip,
            destination_ip,
            source_port,
            destination_port,
            protocol,
        };

        map.entry(key)
            .and_modify(|v| {
                v.total_packet += 1;
                v.total_byte += total_byte;
            })
            .or_insert(ConnectionValue {
                timestamp: packet.timestamp,
                total_packet: 1,
                total_byte,
            });
    }

    map.into_iter()
        .map(|(key, value)| Connection {
            timestamp: value.timestamp,
            interface: key.interface,
            source_ip: key.source_ip.to_string(),
            destination_ip: key.destination_ip.to_string(),
            source_port: key.source_port.map(u32::from),
            destination_port: key.destination_port.map(u32::from),
            protocol: key.protocol.to_string(),
            total_byte: value.total_byte,
            total_packet: value.total_packet,
        })
        .collect()
}

fn extract_ip(net: &Option<NetHeaders>) -> Option<(IpAddr, IpAddr, u16)> {
    match net {
        Some(NetHeaders::Ipv4(h, _)) => {
            let src = IpAddr::V4(Ipv4Addr::from(h.source));
            let dst = IpAddr::V4(Ipv4Addr::from(h.destination));
            Some((src, dst, h.total_len))
        }
        Some(NetHeaders::Ipv6(h, _)) => {
            let src = IpAddr::V6(h.source_addr());
            let dst = IpAddr::V6(h.destination_addr());
            Some((src, dst, 40 + h.payload_length))
        }
        _ => None,
    }
}

fn extract_transport(
    transport: &Option<TransportHeader>,
) -> Option<(Option<u16>, Option<u16>, &'static str)> {
    match transport {
        Some(TransportHeader::Tcp(h)) => {
            Some((Some(h.source_port), Some(h.destination_port), "tcp"))
        }
        Some(TransportHeader::Udp(h)) => {
            Some((Some(h.source_port), Some(h.destination_port), "udp"))
        }
        Some(TransportHeader::Icmpv4(_)) => Some((None, None, "icmpv4")),
        Some(TransportHeader::Icmpv6(_)) => Some((None, None, "icmpv6")),
        None => None,
    }
}

fn is_ignored_interface(name: &str) -> bool {
    // loopback
    if name == "lo" || name == "lo0" {
        return true;
    }
    // virtual/container interfaces
    let virtual_prefixes = ["veth", "docker", "br-", "virbr", "vmnet", "vboxnet", "tun", "tap"];
    virtual_prefixes.iter().any(|p| name.starts_with(p))
}

fn get_packet_headers(packet: &[u8], link_type: i32) -> Option<LaxPacketHeaders<'_>> {
    match link_type {
        // Raw IP, IPv4, IPv6
        12 | 228 | 229 => LaxPacketHeaders::from_ip(packet),
        // NULL, LOOP
        0 | 108 => from_null(packet),
        _ => LaxPacketHeaders::from_ethernet(packet).map_err(LaxHeaderSliceError::Len),
    }
    .handle_err(location!())
    .ok()
}

fn from_null(packet: &[u8]) -> Result<LaxPacketHeaders<'_>, LaxHeaderSliceError> {
    if packet.len() <= 4 {
        return Err(LaxHeaderSliceError::Len(LenError {
            required_len: 4,
            len: packet.len(),
            len_source: LenSource::Slice,
            layer: Layer::Ethernet2Header,
            layer_start_offset: 0,
        }));
    }

    let h = &packet[..4];
    let b = [h[0], h[1], h[2], h[3]];
    let is_valid_af_inet = {
        fn matches(v: u32) -> bool {
            matches!(v, 2 | 24 | 28 | 30)
        }
        matches(u32::from_le_bytes(b)) || matches(u32::from_be_bytes(b))
    };

    if is_valid_af_inet {
        LaxPacketHeaders::from_ip(&packet[4..])
    } else {
        Err(LaxHeaderSliceError::Content(
            HeaderError::UnsupportedIpVersion { version_number: 0 },
        ))
    }
}
