use std::path::PathBuf;

use clap::Args;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, PKCS_ED25519};
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use uuid::Uuid;
use wg_shared::pki::write_secret_file;

mod proto {
    tonic::include_proto!("wallguard.provisioning");
}
use proto::{provisioning_client::ProvisioningClient, EnrollRequest};

#[derive(Args, Debug)]
pub struct EnrollArgs {
    /// Provisioning endpoint, e.g. https://wallguard.example.com:50051
    #[arg(long)]
    pub server: String,

    /// One-time installation code from the web UI or API.
    #[arg(long)]
    pub code: String,

    /// Firewall software on this device.
    #[arg(long, default_value = "none",
          value_parser = ["none", "pfsense", "opnsense", "nftables"])]
    pub firewall: String,

    /// Directory to write device credentials (default: /etc/wallguard).
    #[arg(long, default_value = "/etc/wallguard")]
    pub out_dir: PathBuf,

    /// Path to pinned CA cert for verifying the provisioning server.
    /// If omitted, uses system root CAs (not recommended for production).
    #[arg(long)]
    pub ca_cert: Option<PathBuf>,
}

pub async fn run(args: EnrollArgs) -> anyhow::Result<()> {
    // -----------------------------------------------------------------------
    // 1. Generate an Ed25519 device key pair and a fresh device UUID.
    // -----------------------------------------------------------------------
    let device_id = Uuid::new_v4();
    let key_pair  = KeyPair::generate_for(&PKCS_ED25519)?;

    tracing::debug!(device_id = %device_id, "generated key pair");

    // -----------------------------------------------------------------------
    // 2. Build the CSR: CN=device:<uuid>, O=org:pending
    //    The server will replace the O field with the real org_id.
    // -----------------------------------------------------------------------
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName,       format!("device:{device_id}"));
    dn.push(DnType::OrganizationName, "org:pending".to_string());

    let mut params = CertificateParams::default();
    params.distinguished_name = dn;

    let csr     = params.serialize_request(&key_pair)?;
    let csr_pem = csr.pem()?;

    // -----------------------------------------------------------------------
    // 3. Connect to the Provisioning gRPC service with server-cert verification.
    // -----------------------------------------------------------------------
    let tls = build_tls_config(args.ca_cert.as_deref(), &args.server)?;
    let channel = Channel::from_shared(args.server.clone())?
        .tls_config(tls)?
        .connect()
        .await?;

    let mut client = ProvisioningClient::new(channel);

    // -----------------------------------------------------------------------
    // 4. Enroll.
    // -----------------------------------------------------------------------
    tracing::info!("enrolling device {device_id} …");

    let resp = client
        .enroll(EnrollRequest {
            installation_code: args.code.clone(),
            csr_pem:           csr_pem.clone(),
            firewall_kind:     args.firewall.clone(),
            agent_version:     env!("CARGO_PKG_VERSION").to_string(),
        })
        .await?
        .into_inner();

    tracing::info!(device_id = %resp.device_id, "enrollment successful");

    // -----------------------------------------------------------------------
    // 5. Write output files.
    // -----------------------------------------------------------------------
    std::fs::create_dir_all(&args.out_dir)?;

    // device.key — mode 0600 (secret key)
    let key_path = args.out_dir.join("device.key");
    write_secret_file(&key_path, key_pair.serialize_pem().as_bytes())?;

    // device.crt — mode 0644
    let crt_path = args.out_dir.join("device.crt");
    std::fs::write(&crt_path, resp.device_cert_pem.as_bytes())?;

    // ca.crt — mode 0644
    let ca_path = args.out_dir.join("ca.crt");
    std::fs::write(&ca_path, resp.ca_cert_pem.as_bytes())?;

    // config.toml
    let cfg_path = args.out_dir.join("config.toml");
    let cfg_toml = build_config_toml(
        &resp.device_id,
        &resp.server_name,
        &args.out_dir,
        &args.firewall,
    );
    std::fs::write(&cfg_path, cfg_toml.as_bytes())?;

    println!("Enrolled successfully.");
    println!("  Device ID  : {}", resp.device_id);
    println!("  Key        : {}", key_path.display());
    println!("  Cert       : {}", crt_path.display());
    println!("  CA cert    : {}", ca_path.display());
    println!("  Config     : {}", cfg_path.display());
    println!();
    println!("Start the agent: wg-agent --config {}", cfg_path.display());

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_tls_config(
    ca_cert_path: Option<&std::path::Path>,
    server_url:   &str,
) -> anyhow::Result<ClientTlsConfig> {
    // Extract the hostname from the URL for SNI.
    let host = url_host(server_url).unwrap_or("localhost");

    let mut tls = ClientTlsConfig::new().domain_name(host);

    if let Some(path) = ca_cert_path {
        let pem = std::fs::read_to_string(path)?;
        tls = tls.ca_certificate(Certificate::from_pem(pem));
    }

    Ok(tls)
}

fn url_host(url: &str) -> Option<&str> {
    // Very small parser: strip scheme, take everything before the next '/' or ':'.
    let after_scheme = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    let host_port    = after_scheme.split('/').next()?;
    // Strip port if present.
    let host = host_port.rsplit_once(':').map(|(h, _)| h).unwrap_or(host_port);
    Some(host)
}

fn build_config_toml(
    device_id:  &str,
    server_name: &str,
    out_dir:    &PathBuf,
    firewall:   &str,
) -> String {
    format!(
        r#"# WallGuard agent configuration
# Generated by `wg-cli enroll`

[device]
id            = "{device_id}"
firewall_kind = "{firewall}"

[server]
name          = "{server_name}"
grpc_port     = 50052
quic_port     = 7777
tcp_port      = 7778

[tls]
device_key    = "{key}"
device_cert   = "{cert}"
ca_cert       = "{ca}"

[agent]
heartbeat_interval_s = 10
reconnect_base_s     = 1
reconnect_max_s      = 300
"#,
        key  = out_dir.join("device.key").display(),
        cert = out_dir.join("device.crt").display(),
        ca   = out_dir.join("ca.crt").display(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_host_extraction() {
        assert_eq!(url_host("https://server.example.com:50051"), Some("server.example.com"));
        assert_eq!(url_host("https://localhost:50051"),           Some("localhost"));
        assert_eq!(url_host("https://127.0.0.1:50051"),           Some("127.0.0.1"));
    }

    #[test]
    fn csr_has_correct_dn() {
        let device_id = Uuid::new_v4();
        let key_pair  = KeyPair::generate_for(&PKCS_ED25519).unwrap();

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName,       format!("device:{device_id}"));
        dn.push(DnType::OrganizationName, "org:pending".to_string());

        let mut params = CertificateParams::default();
        params.distinguished_name = dn;

        let csr     = params.serialize_request(&key_pair).unwrap();
        let csr_pem = csr.pem().unwrap();

        assert!(csr_pem.contains("CERTIFICATE REQUEST"));
        assert!(csr_pem.len() > 100);
    }
}
