use sysinfo::Disks;
use crate::utilities;

fn collect_disks_info() -> Option<String> {
    let disks = Disks::new_with_refreshed_list();

    let mut parts = Vec::new();
    for disk in &disks {
        parts.push(format!("{}", disk.name().to_string_lossy()));
        parts.push(format!("{}", disk.kind().to_string()));
        parts.push(format!("{}", disk.file_system().to_string_lossy()));
        parts.push(format!("{}", disk.file_system().to_string_lossy()));
        parts.push(format!("{}", disk.total_space()));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

pub fn disks_fingerprint() -> Option<String> {
    let raw_id = collect_disks_info()?;
    Some(utilities::hash::sha256_digest_hex(&raw_id))
}
