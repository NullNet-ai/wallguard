use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ptr;

use winapi::shared::iprtrmib::TCP_TABLE_OWNER_PID_ALL;
use winapi::shared::minwindef::FALSE;
use winapi::shared::ntdef::ULONG;
use winapi::shared::tcpmib::{
    MIB_TCP_STATE_LISTEN, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID, MIB_TCPROW_OWNER_PID,
    MIB_TCPTABLE_OWNER_PID,
};
use winapi::shared::winerror::NO_ERROR;
use winapi::shared::ws2def::{AF_INET, AF_INET6};
use winapi::um::iphlpapi::GetExtendedTcpTable;

fn get_tcp_table(af: ULONG) -> io::Result<Vec<u8>> {
    let mut size: u32 = 0;

    unsafe {
        GetExtendedTcpTable(
            ptr::null_mut(),
            &mut size,
            FALSE,
            af,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        let mut buffer = vec![0u8; size as usize];

        let ret = GetExtendedTcpTable(
            buffer.as_mut_ptr() as _,
            &mut size,
            FALSE,
            af,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if ret != NO_ERROR {
            return Err(io::Error::last_os_error());
        }

        Ok(buffer)
    }
}

pub(crate) fn tcp_sockets() -> io::Result<Vec<(SocketAddr, u32)>> {
    unsafe {
        let buffer = get_tcp_table(AF_INET as ULONG)?;
        let table_ptr = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
        let num_entries = (*table_ptr).dwNumEntries as usize;

        let rows = std::slice::from_raw_parts(
            &(*table_ptr).table as *const MIB_TCPROW_OWNER_PID,
            num_entries,
        );

        let values = rows
            .iter()
            .filter(|row| row.dwState == MIB_TCP_STATE_LISTEN)
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

pub(crate) fn tcp6_sockets() -> io::Result<Vec<(SocketAddr, u32)>> {
    unsafe {
        let buffer = get_tcp_table(AF_INET6 as ULONG)?;
        let table_ptr = buffer.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID;
        let num_entries = (*table_ptr).dwNumEntries as usize;

        let rows = std::slice::from_raw_parts(
            &(*table_ptr).table as *const MIB_TCP6ROW_OWNER_PID,
            num_entries,
        );

        let values = rows
            .iter()
            .filter(|row| row.dwState == MIB_TCP_STATE_LISTEN)
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
