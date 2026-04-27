use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::Result;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};

/// Generate a fresh ECDSA keypair and a CSR with `device_id` as the CN.
/// Returns `(key_pem, csr_pem)`.
pub fn generate_csr(device_id: &str) -> Result<(String, String)> {
    let key = KeyPair::generate()?;

    let mut params = CertificateParams::default();
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, device_id);
    params.distinguished_name = dn;

    let csr     = params.serialize_request(&key)?;
    let csr_pem = csr.pem()?;
    let key_pem = key.serialize_pem();

    Ok((key_pem, csr_pem))
}

/// Atomically install a new certificate set.
///
/// Writes each file to a `.new` sibling path first, `fsync`s, then
/// renames over the live path — the agent is never left with a partial
/// cert set on disk if the process is killed mid-write.
pub fn install_cert(
    cert_pem:  &str,
    ca_pem:    &str,
    key_pem:   &str,
    cert_path: &Path,
    ca_path:   &Path,
    key_path:  &Path,
) -> Result<()> {
    let cert_tmp = tmp_path(cert_path);
    let ca_tmp   = tmp_path(ca_path);
    let key_tmp  = tmp_path(key_path);

    write_and_sync(&cert_tmp, cert_pem)?;
    write_and_sync(&ca_tmp,   ca_pem)?;
    write_and_sync(&key_tmp,  key_pem)?;

    std::fs::rename(&cert_tmp, cert_path)?;
    std::fs::rename(&ca_tmp,   ca_path)?;
    std::fs::rename(&key_tmp,  key_path)?;

    Ok(())
}

fn tmp_path(p: &Path) -> PathBuf {
    let mut s = p.as_os_str().to_os_string();
    s.push(".new");
    PathBuf::from(s)
}

fn write_and_sync(path: &Path, data: &str) -> Result<()> {
    let mut f = std::fs::File::create(path)?;
    f.write_all(data.as_bytes())?;
    f.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_csr_produces_valid_pem() {
        let (key_pem, csr_pem) = generate_csr("test-device-abc123").unwrap();
        assert!(key_pem.contains("-----BEGIN PRIVATE KEY-----"));
        assert!(csr_pem.contains("-----BEGIN CERTIFICATE REQUEST-----"));
    }

    #[test]
    fn install_cert_writes_and_renames_atomically() {
        let dir = tempfile::tempdir().unwrap();

        let cert_path = dir.path().join("device.pem");
        let ca_path   = dir.path().join("ca.pem");
        let key_path  = dir.path().join("device.key");

        // Generate real material so the install round-trips cleanly.
        let key = KeyPair::generate().unwrap();
        let params = CertificateParams::default();
        let cert   = params.self_signed(&key).unwrap();
        let cert_pem = cert.pem();
        let key_pem  = key.serialize_pem();

        install_cert(&cert_pem, &cert_pem, &key_pem, &cert_path, &ca_path, &key_path).unwrap();

        assert_eq!(std::fs::read_to_string(&cert_path).unwrap(), cert_pem);
        assert_eq!(std::fs::read_to_string(&key_path).unwrap(),  key_pem);

        // Temporary .new files must not linger.
        assert!(!tmp_path(&cert_path).exists());
        assert!(!tmp_path(&ca_path).exists());
        assert!(!tmp_path(&key_path).exists());
    }
}
