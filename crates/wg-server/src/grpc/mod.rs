pub mod control;
pub mod data;
pub mod provisioning;

use uuid::Uuid;

/// Extract the device UUID from the `CN=device:<uuid>` field of the
/// mTLS peer certificate carried in any tonic `Request<T>`.
pub fn extract_device_id<T>(request: &tonic::Request<T>) -> Option<Uuid> {
    let certs = request.peer_certs()?;
    let der   = certs.first()?.as_ref();
    extract_device_id_from_der(der)
}

fn extract_device_id_from_der(der: &[u8]) -> Option<Uuid> {
    use x509_parser::prelude::*;
    let (_, cert) = X509Certificate::from_der(der).ok()?;
    let cn = cert.subject().iter_common_name().next()?.as_str().ok()?;
    Uuid::parse_str(cn.strip_prefix("device:")?).ok()
}
