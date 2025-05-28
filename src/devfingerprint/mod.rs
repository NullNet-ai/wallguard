use disk::disks_fingerprint;
use system::system_fingerprint;

use crate::utilities;

mod disk;
mod system;

pub fn devfingerprint() -> Option<String> {
    let mut parts = Vec::new();

    parts.push(disks_fingerprint()?);
    parts.push(system_fingerprint()?);

    let value = parts.join("");
    Some(utilities::hash::sha256_digest_hex(&value))
}
