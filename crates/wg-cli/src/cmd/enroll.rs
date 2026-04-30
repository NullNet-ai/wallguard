use std::path::PathBuf;

use clap::Args;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, PKCS_ED25519};
use tonic::transport::Channel;
#[cfg(not(debug_assertions))]
use tonic::transport::{Certificate, ClientTlsConfig};
use uuid::Uuid;
use wg_shared::pki::write_secret_file;

// ---------------------------------------------------------------------------
// Debug-only: skip server cert verification (self-signed dev PKI)
// ---------------------------------------------------------------------------

#[cfg(debug_assertions)]
mod debug_tls {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use hyper_util::rt::TokioIo;
    use tokio::net::TcpStream;
    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use rustls::{DigitallySignedStruct, Error, SignatureScheme};

    #[derive(Debug)]
    pub struct NoCertVerifier;

    impl ServerCertVerifier for NoCertVerifier {
        fn verify_server_cert(&self, _: &CertificateDer<'_>, _: &[CertificateDer<'_>],
            _: &ServerName<'_>, _: &[u8], _: UnixTime) -> Result<ServerCertVerified, Error> {
            Ok(ServerCertVerified::assertion())
        }
        fn verify_tls12_signature(&self, _: &[u8], _: &CertificateDer<'_>,
            _: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn verify_tls13_signature(&self, _: &[u8], _: &CertificateDer<'_>,
            _: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            rustls::crypto::aws_lc_rs::default_provider()
                .signature_verification_algorithms.supported_schemes()
        }
    }

    pub type Io = TokioIo<tokio_rustls::client::TlsStream<TcpStream>>;

    #[derive(Clone)]
    pub struct Connector {
        tls:         tokio_rustls::TlsConnector,
        server_name: rustls_pki_types::ServerName<'static>,
    }

    impl Connector {
        pub fn new(server_name: &str) -> anyhow::Result<Self> {
            let mut cfg = rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoCertVerifier))
                .with_no_client_auth();
            cfg.alpn_protocols.push(b"h2".to_vec());
            let server_name = rustls_pki_types::ServerName::try_from(server_name.to_string())
                .map_err(|e| anyhow::anyhow!("invalid server name '{server_name}': {e}"))?;
            Ok(Self { tls: tokio_rustls::TlsConnector::from(Arc::new(cfg)), server_name })
        }
    }

    impl tower::Service<tonic::transport::Uri> for Connector {
        type Response = Io;
        type Error   = anyhow::Error;
        type Future  = Pin<Box<dyn Future<Output = Result<Io, anyhow::Error>> + Send>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, uri: tonic::transport::Uri) -> Self::Future {
            let tls         = self.tls.clone();
            let server_name = self.server_name.clone();
            Box::pin(async move {
                let host = uri.host().unwrap_or_default();
                let port = uri.port_u16().unwrap_or(443);
                let tcp  = TcpStream::connect((host, port)).await
                    .map_err(|e| anyhow::anyhow!("TCP connect {host}:{port}: {e}"))?;
                Ok(TokioIo::new(tls.connect(server_name, tcp).await
                    .map_err(|e| anyhow::anyhow!("TLS handshake: {e}"))?))
            })
        }
    }
}

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

    /// Override the server address written to config.toml.
    /// Useful when the server is behind NAT or Docker and its advertised name
    /// isn't reachable from this host (e.g. --connect-address 127.0.0.1).
    #[arg(long)]
    pub connect_address: Option<String>,
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
    // tonic requires https:// to negotiate TLS; normalize grpc:// aliases.
    let endpoint_url = normalize_scheme(&args.server);

    #[cfg(debug_assertions)]
    let channel = {
        tracing::warn!("TLS server certificate verification DISABLED — debug build only");
        let host = url_host(&endpoint_url).unwrap_or("localhost");
        let connector = debug_tls::Connector::new(host)?;
        // Use http:// so tonic doesn't require its own TLS config — our
        // connector handles TLS internally.
        let http_url = endpoint_url.replacen("https://", "http://", 1);
        Channel::from_shared(http_url)?
            .connect_with_connector(connector)
            .await?
    };
    #[cfg(not(debug_assertions))]
    let channel = {
        let tls = build_tls_config(args.ca_cert.as_deref(), &endpoint_url)?;
        Channel::from_shared(endpoint_url)?
            .tls_config(tls)?
            .connect()
            .await?
    };

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
    let connect_addr = args.connect_address.as_deref()
        .or_else(|| url_host(&args.server))
        .unwrap_or("localhost");
    let cfg_toml = build_config_toml(
        &resp.device_id,
        connect_addr,
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

#[cfg(not(debug_assertions))]
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

/// Convert grpc:// / grpc+tls:// to https:// so tonic uses TLS.
fn normalize_scheme(url: &str) -> String {
    for prefix in &["grpc+tls://", "grpc://"] {
        if let Some(rest) = url.strip_prefix(prefix) {
            return format!("https://{rest}");
        }
    }
    url.to_owned()
}

fn url_host(url: &str) -> Option<&str> {
    // Strip any known scheme, then take the host before '/' or ':'.
    let after_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("grpc+tls://"))
        .or_else(|| url.strip_prefix("grpc://"))?;
    let host_port = after_scheme.split('/').next()?;
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
