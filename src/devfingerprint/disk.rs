use crate::utilities;
use sysinfo::Disks;

fn collect_disks_info() -> Option<String> {
    let disks = Disks::new_with_refreshed_list();

    let parts: Vec<String> = disks
        .iter()
        .map(|disk| {
            format!(
                "{}|{}|{}|{}",
                disk.name().to_string_lossy(),
                disk.kind().to_string(),
                disk.file_system().to_string_lossy(),
                disk.total_space()
            )
        })
        .collect();

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

pub fn disks_fingerprint() -> Option<String> {
    collect_disks_info().map(|raw| utilities::hash::sha256_digest_hex(&raw))
}
