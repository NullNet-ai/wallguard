use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ptr;

use winapi::shared::iprtrmib::UDP_TABLE_OWNER_PID;
use winapi::shared::minwindef::FALSE;
use winapi::shared::udpmib::{
    MIB_UDP6ROW_OWNER_PID, MIB_UDP6TABLE_OWNER_PID, MIB_UDPROW_OWNER_PID, MIB_UDPTABLE_OWNER_PID,
};
use winapi::shared::ws2def::{AF_INET, AF_INET6};
use winapi::um::iphlpapi::GetExtendedUdpTable;

fn get_udp_table(af: u32) -> io::Result<Vec<u8>> {
    let mut size: u32 = 0;
    unsafe {
        GetExtendedUdpTable(
            ptr::null_mut(),
            &mut size,
            FALSE,
            af,
            UDP_TABLE_OWNER_PID,
            0,
        );

        let mut buffer = vec![0u8; size as usize];

        let ret = GetExtendedUdpTable(
            buffer.as_mut_ptr() as _,
            &mut size,
            FALSE,
            af,
            UDP_TABLE_OWNER_PID,
            0,
        );

        if ret != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(buffer)
    }
}

pub(crate) fn udp_sockets() -> io::Result<Vec<(SocketAddr, u32)>> {
    unsafe {
        let buffer = get_udp_table(AF_INET as u32)?;
        let table_ptr = buffer.as_ptr() as *const MIB_UDPTABLE_OWNER_PID;
        let count = (*table_ptr).dwNumEntries as usize;

        let rows =
            std::slice::from_raw_parts(&(*table_ptr).table as *const MIB_UDPROW_OWNER_PID, count);

        let values = rows
            .iter()
            .map(|row| {
                let addr = Ipv4Addr::new(
                    ((row.dwLocalAddr >> 00) & 0xff) as u8,
                    ((row.dwLocalAddr >> 08) & 0xff) as u8,
                    ((row.dwLocalAddr >> 16) & 0xff) as u8,
                    ((row.dwLocalAddr >> 24) & 0xff) as u8,
                );
                let port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);

                (
                    SocketAddr::V4(SocketAddrV4::new(addr, port)),
                    row.dwOwningPid,
                )
            })
            .collect::<Vec<_>>();

        Ok(values)
    }
}

pub(crate) fn udp6_sockets() -> io::Result<Vec<(SocketAddr, u32)>> {
    unsafe {
        let buffer = get_udp_table(AF_INET6 as u32)?;
        let table_ptr = buffer.as_ptr() as *const MIB_UDP6TABLE_OWNER_PID;
        let count = (*table_ptr).dwNumEntries as usize;

        let rows =
            std::slice::from_raw_parts(&(*table_ptr).table as *const MIB_UDP6ROW_OWNER_PID, count);

        let values = rows
            .iter()
            .map(|row| {
                let octets: [u8; 16] = std::mem::transmute(row.ucLocalAddr);
                let addr = Ipv6Addr::from_octets(octets);
                let port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);

                (
                    SocketAddr::V6(SocketAddrV6::new(addr, port, 0, row.dwLocalScopeId)),
                    row.dwOwningPid,
                )
            })
            .collect::<Vec<_>>();

        Ok(values)
    }
}
