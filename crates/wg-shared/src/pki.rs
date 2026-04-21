/// Parse the device UUID from the CN field of a PEM-encoded certificate.
/// Expected CN format: `device:<uuid>`.
///
/// Returns `None` if the cert cannot be parsed or the CN does not match the
/// expected format.  Full implementation wired up in Phase 3 (PKI/CA) once
/// `x509-parser` is added as a native dep of `wg-server`.  The stub is
/// sufficient for Phase 0-2 compilation.
pub fn parse_device_id_from_cert(_cert_pem: &str) -> Option<uuid::Uuid> {
    None
}

/// Write secret material to a file with mode 0600.
///
/// The permission is set at `open()` time — before any bytes are written —
/// so a concurrent reader can never observe a window where the file exists
/// with world-readable permissions.
///
/// Only available on Unix targets.  Not compiled for `wasm32`.
#[cfg(unix)]
pub fn write_secret_file(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    use std::{
        fs::OpenOptions,
        io::Write,
        os::unix::fs::OpenOptionsExt,
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(data)?;
    file.sync_all()
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;

    #[test]
    fn write_secret_creates_file_with_correct_mode() {
        use std::os::unix::fs::PermissionsExt;
        let dir  = tempfile::tempdir().unwrap();
        let path = dir.path().join("secret.key");

        write_secret_file(&path, b"hunter2").unwrap();

        let meta = std::fs::metadata(&path).unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "expected mode 0600, got {mode:04o}");
        assert_eq!(std::fs::read(&path).unwrap(), b"hunter2");
    }
}
