use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use winapi::um::handleapi::CloseHandle;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use winapi::um::winnt::HANDLE;

pub fn snapshot_processes() -> std::io::Result<HashMap<u32, String>> {
    let mut map = HashMap::<u32, String>::new();

    unsafe {
        let snapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

        if snapshot.is_null() {
            return Err(std::io::Error::last_os_error());
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry as *mut _) == 0 {
            CloseHandle(snapshot);
            return Err(std::io::Error::last_os_error());
        }

        loop {
            let len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = OsString::from_wide(&entry.szExeFile[..len])
                .to_string_lossy()
                .into_owned();

            map.insert(entry.th32ProcessID, name);

            if Process32NextW(snapshot, &mut entry as *mut _) == 0 {
                break;
            }
        }

        CloseHandle(snapshot);
    }

    Ok(map)
}
