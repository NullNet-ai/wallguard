use crate::utilities;
use std::ffi::CString;
use std::io;
use std::process::Command;
use std::ptr;

fn get_serial_from_device(device: &str) -> io::Result<Option<String>> {
    let output = Command::new("camcontrol")
        .arg("identify")
        .arg(device)
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let line_lower = line.to_lowercase();

        if line_lower.contains("serial") && line_lower.contains("number") {
            if let Some(pos) = line.find(':') {
                let serial = line[pos + 1..].trim();
                if !serial.is_empty() {
                    return Ok(Some(serial.to_string()));
                }
            } else {
                let parts: Vec<_> = line.split_whitespace().collect();
                if let Some(last) = parts.last() {
                    return Ok(Some(last.to_string()));
                }
            }
        }
    }

    Ok(None)
}

fn get_disknames() -> io::Result<Vec<String>> {
    let oid = CString::new("kern.disks").unwrap();
    let mut size: libc::size_t = 0;

    let ret =
        unsafe { libc::sysctlbyname(oid.as_ptr(), ptr::null_mut(), &mut size, ptr::null(), 0) };

    if ret != 0 {
        return Err(io::Error::last_os_error());
    }

    let mut buf = vec![0u8; size];

    let ret = unsafe {
        libc::sysctlbyname(
            oid.as_ptr(),
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            ptr::null(),
            0,
        )
    };

    if ret != 0 {
        return Err(io::Error::last_os_error());
    }

    let s = String::from_utf8_lossy(&buf);
    Ok(s.split_whitespace()
        .map(|s| s.trim_end_matches('\0').to_string())
        .collect())
}

pub fn disks_fingerprint() -> Option<String> {
    // Get disks and serials as Vec<(String, Option<String>)>
    let disks = match disks_fingerprint_raw() {
        Ok(disks) => disks,
        Err(_) => return None,
    };

    // Filter out disks without serial or replace None with empty string
    let filtered: Vec<_> = disks
        .into_iter()
        .filter_map(|(devnode, serial_opt)| serial_opt.map(|serial| (devnode, serial)))
        .collect();

    if filtered.is_empty() {
        return None;
    }

    let raw = filtered
        .iter()
        .map(|(devnode, serial)| format!("{}|{}", devnode, serial))
        .collect::<Vec<_>>()
        .join("\n");

    Some(utilities::hash::sha256_digest_hex(&raw))
}
