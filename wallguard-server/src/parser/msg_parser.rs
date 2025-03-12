use crate::parser::models::transport::header::TransportHeader;
use crate::proto::wallguard::Packets;
use etherparse::err::ip::{HeaderError, LaxHeaderSliceError};
use etherparse::err::{Layer, LenError};
use etherparse::{LaxPacketHeaders, LenSource};
use nullnet_liberror::{location, ErrorHandler, Location};
use nullnet_libtoken::Token;

use super::{
    models::{ether::header::EthernetHeader, ip::header::IpHeader},
    parsed_message::{ParsedMessage, ParsedRecord},
};

pub fn parse_message(message: Packets, token: &Token) -> ParsedMessage {
    let mut records = Vec::new();

    for packet in message.packets {
        let interface_name = packet.interface;
        let link_type = packet.link_type;
        let timestamp = packet.timestamp;

        if let Some(headers) = get_packet_headers(&packet.data, link_type) {
            let ethernet_header = EthernetHeader::from_etherparse(headers.link);
            if let Some(ip_header) = IpHeader::from_etherparse(headers.net) {
                if let Some(transport_header) = TransportHeader::from_etherparse(headers.transport)
                {
                    let total_length = ethernet_header.as_ref().map_or(0, |_| 12)
                        + ip_header.ip_header_length as u16
                        + ip_header.payload_length;
                    records.push(ParsedRecord {
                        device_id: token.account.device.id.clone(),
                        interface_name,
                        total_length,
                        timestamp,
                        ethernet_header,
                        ip_header,
                        transport_header,
                    });
                }
            }
        }
    }

    ParsedMessage { records }
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
