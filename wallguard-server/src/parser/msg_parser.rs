use super::{ip_header::IpHeader, parsed_message::ParsedMessage};
use crate::parser::connections_map::{ConnectionKey, ConnectionValue, ConnectionsMap};
use crate::parser::transport_header::TransportHeader;
use crate::proto::wallguard::Packets;
use etherparse::err::ip::{HeaderError, LaxHeaderSliceError};
use etherparse::err::{Layer, LenError};
use etherparse::{LaxPacketHeaders, LenSource, LinkHeader};
use nullnet_liberror::{ErrorHandler, Location, location};
use nullnet_libipinfo::get_ip_to_lookup;
use nullnet_libtoken::Token;
use std::net::IpAddr;
use std::sync::mpsc::Sender;

pub fn parse_message(
    message: Packets,
    token: &Token,
    ip_info_tx: &Sender<Option<IpAddr>>,
) -> ParsedMessage {
    let mut map = ConnectionsMap::new();
    for packet in message.packets {
        let link_type = packet.link_type;
        if let Some(headers) = get_packet_headers(&packet.data, link_type) {
            if let Some(ip_header) = IpHeader::from_etherparse(headers.net) {
                if let Some(transport_header) = TransportHeader::from_etherparse(headers.transport)
                {
                    let device_id = token.account.device.id.clone();
                    let interface_name = packet.interface;
                    let has_eth = matches!(headers.link, Some(LinkHeader::Ethernet2(_)));
                    let bytes = 14 * usize::from(has_eth) + usize::from(ip_header.packet_length);
                    let source_ip = ip_header.source_ip;
                    let destination_ip = ip_header.destination_ip;

                    let key =
                        ConnectionKey::new(device_id, interface_name, ip_header, transport_header);

                    map.connections
                        .entry(key)
                        .and_modify(|v| {
                            v.update(bytes);
                        })
                        .or_insert({
                            let timestamp = packet.timestamp;
                            let remote_ip = get_ip_to_lookup(source_ip, destination_ip);
                            let _ = ip_info_tx.send(remote_ip);
                            ConnectionValue::new(timestamp, bytes, remote_ip)
                        });
                }
            }
        }
    }
    map.into_parsed_message()
}

fn get_packet_headers(packet: &[u8], link_type: i32) -> Option<LaxPacketHeaders> {
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

fn from_null(packet: &[u8]) -> Result<LaxPacketHeaders, LaxHeaderSliceError> {
    if packet.len() <= 4 {
        return Err(LaxHeaderSliceError::Len(LenError {
            required_len: 4,
            len: packet.len(),
            len_source: LenSource::Slice,
            layer: Layer::Ethernet2Header,
            layer_start_offset: 0,
        }));
    }

    let is_valid_af_inet = {
        // based on https://wiki.wireshark.org/NullLoopback.md (2023-12-31)
        fn matches(value: u32) -> bool {
            match value {
                // 2 = IPv4 on all platforms
                // 24, 28, or 30 = IPv6 depending on platform
                2 | 24 | 28 | 30 => true,
                _ => false,
            }
        }
        let h = &packet[..4];
        let b = [h[0], h[1], h[2], h[3]];
        // check both big endian and little endian representations
        // as some OS'es use native endianess and others use big endian
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
