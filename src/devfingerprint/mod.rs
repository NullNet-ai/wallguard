use _disk::disks_fingerprint;
use system::system_fingerprint;

use crate::utilities;

mod disk;
mod _disk;
mod system;

pub fn devfingerprint() -> Option<String> {
    let mut parts = Vec::new();

    if let Some(df) = disks_fingerprint() {
        parts.push(df);
    }

    if let Some(sf) = system_fingerprint() {
        parts.push(sf);
    }

    if parts.is_empty() {
        None
    } else {
        let value = parts.join("");
        Some(utilities::hash::sha256_digest_hex(&value))
    }
}
