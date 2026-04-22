use rcgen::{
    CertificateParams, CertificateSigningRequestParams, DnType, DnValue, KeyPair,
};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum CaError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("rcgen: {0}")]
    Rcgen(#[from] rcgen::Error),
    #[error("invalid CSR subject: {0}")]
    InvalidSubject(String),
}

/// Loaded Intermediate CA — used to sign device CSRs during provisioning.
///
/// Not `Clone`; wrap in `Arc<Ca>` when sharing across axum handlers.
pub struct Ca {
    cert: rcgen::Certificate,
    key:  KeyPair,
}

impl Ca {
    /// Load the Intermediate CA from PEM strings.
    ///
    /// Both PEMs come from files on disk (mounted secrets in production,
    /// `dev-certs/ca.crt` + `dev-certs/ca.key` in development).
    pub fn load_pem(cert_pem: &str, key_pem: &str) -> Result<Self, CaError> {
        let key    = KeyPair::from_pem(key_pem)?;
        let params = CertificateParams::from_ca_cert_pem(cert_pem)?;
        // Reconstruct the Certificate for use as an issuer.  Only the DN and
        // key usage fields matter for signing; the exact cert bytes do not.
        let cert = params.self_signed(&key)?;
        Ok(Self { cert, key })
    }

    /// Sign a device CSR and return `(cert_pem, device_id, org_id)`.
    ///
    /// Validates that the CSR subject contains:
    ///   `CN=device:<uuid>`
    ///   `O=org:<uuid>`
    pub fn sign_csr(&self, csr_pem: &str) -> Result<(String, Uuid, Uuid), CaError> {
        let csr = CertificateSigningRequestParams::from_pem(csr_pem)?;
        let (device_id, org_id) = extract_ids_from_dn(&csr.params.distinguished_name)?;

        let signed = csr.signed_by(&self.cert, &self.key)?;
        Ok((signed.pem(), device_id, org_id))
    }
}

/// Extract `device:<uuid>` from CN and `org:<uuid>` from O.
fn extract_ids_from_dn(
    dn: &rcgen::DistinguishedName,
) -> Result<(Uuid, Uuid), CaError> {
    let mut device_id: Option<Uuid> = None;
    let mut org_id: Option<Uuid>    = None;

    for (dn_type, value) in dn.iter() {
        let s = dn_value_str(value).ok_or_else(|| {
            CaError::InvalidSubject("non-UTF8 DN value".into())
        })?;

        match dn_type {
            t if *t == DnType::CommonName => {
                let raw = s.strip_prefix("device:").ok_or_else(|| {
                    CaError::InvalidSubject(format!("CN must be 'device:<uuid>', got '{s}'"))
                })?;
                device_id = Some(Uuid::parse_str(raw).map_err(|_| {
                    CaError::InvalidSubject(format!("CN UUID invalid: '{raw}'"))
                })?);
            }
            t if *t == DnType::OrganizationName => {
                let raw = s.strip_prefix("org:").ok_or_else(|| {
                    CaError::InvalidSubject(format!("O must be 'org:<uuid>', got '{s}'"))
                })?;
                org_id = Some(Uuid::parse_str(raw).map_err(|_| {
                    CaError::InvalidSubject(format!("O UUID invalid: '{raw}'"))
                })?);
            }
            _ => {}
        }
    }

    let device_id =
        device_id.ok_or_else(|| CaError::InvalidSubject("CN (device id) missing".into()))?;
    let org_id =
        org_id.ok_or_else(|| CaError::InvalidSubject("O (org id) missing".into()))?;

    Ok((device_id, org_id))
}

fn dn_value_str(v: &DnValue) -> Option<&str> {
    match v {
        DnValue::Utf8String(s)      => Some(s.as_str()),
        DnValue::PrintableString(s) => Some(s.as_ref()),
        DnValue::Ia5String(s)       => Some(s.as_ref()),
        // BmpString / TeletexString / UniversalString are exotic and not used
        // in normal device CSRs.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::{CertificateParams, DistinguishedName, KeyPair};

    fn make_test_ca() -> Ca {
        let ca_key    = KeyPair::generate().unwrap();
        let mut params = CertificateParams::default();
        params.is_ca   = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        let ca_cert = params.self_signed(&ca_key).unwrap();
        Ca { cert: ca_cert, key: ca_key }
    }

    fn make_csr(cn: impl Into<String>, org: impl Into<String>) -> String {
        let device_key = KeyPair::generate().unwrap();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, cn.into());
        dn.push(DnType::OrganizationName, org.into());
        let mut params = CertificateParams::default();
        params.distinguished_name = dn;
        let csr = params.serialize_request(&device_key).unwrap();
        csr.pem().unwrap()
    }

    #[test]
    fn sign_valid_csr() {
        let ca         = make_test_ca();
        let device_id  = Uuid::new_v4();
        let org_id     = Uuid::new_v4();
        let csr_pem    = make_csr(format!("device:{device_id}"), format!("org:{org_id}"));

        let (cert_pem, got_device, got_org) = ca.sign_csr(&csr_pem).unwrap();

        assert_eq!(got_device, device_id);
        assert_eq!(got_org, org_id);
        assert!(cert_pem.contains("CERTIFICATE"));
    }

    #[test]
    fn rejects_missing_cn() {
        let ca      = make_test_ca();
        let csr_pem = make_csr(format!("wrong:value"), format!("org:{}", Uuid::new_v4()));
        assert!(matches!(ca.sign_csr(&csr_pem), Err(CaError::InvalidSubject(_))));
    }

    #[test]
    fn rejects_missing_org() {
        let ca      = make_test_ca();
        let csr_pem = make_csr(
            format!("device:{}", Uuid::new_v4()),
            format!("notanorg"),
        );
        assert!(matches!(ca.sign_csr(&csr_pem), Err(CaError::InvalidSubject(_))));
    }
}
