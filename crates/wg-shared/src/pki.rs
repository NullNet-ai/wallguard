use std::{fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt, path::Path};

/// Parse the device UUID from the CN of a PEM-encoded certificate.
/// Expected CN format: `device:<uuid>`.
pub fn parse_device_id_from_cert(cert_pem: &str) -> Option<uuid::Uuid> {
    // Full implementation in Phase 3 (PKI/CA).
    // Stub: returns None until rcgen-based parsing is wired up.
    let _ = cert_pem;
    None
}

/// Write secret material to a file with mode 0600.
/// The mode is set at open() time, before any data is written.
pub fn write_secret_file(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(data)?;
    file.sync_all()?;
    Ok(())
}
