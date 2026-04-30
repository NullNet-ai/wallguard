use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;

use crate::config::Config;
use crate::platform::{TargetOs, TARGET_OS};
use crate::proto::control::HttpService;

const PROBE_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_CONCURRENT: usize = 10;

/// Ports that are clearly not HTTP — never probed.
const SKIP_PORTS: &[u16] = &[
    21, 22, 23, 25, 53, 110, 143, 389, 445, 465, 587, 636,
    3306, 5432, 5900, 6379, 11211, 27017,
];

/// Scan localhost for listening HTTP/HTTPS services.
///
/// Phase 1: enumerate listening TCP ports from the OS without network I/O.
/// Phase 2: probe each candidate with a short-timeout HTTP request; try
///          plain HTTP first, then TLS (accepting any certificate).
pub async fn scan(config: &Config) -> Vec<HttpService> {
    let ports = match TARGET_OS {
        TargetOs::Linux   => listen_ports_linux(),
        TargetOs::FreeBsd => listen_ports_freebsd(),
        _                 => return vec![],
    };

    let mut skip: HashSet<u16> = SKIP_PORTS.iter().copied().collect();
    skip.insert(config.server.grpc_port);
    skip.insert(config.server.quic_port);
    skip.insert(config.server.tcp_port);
    skip.insert(config.agent.ssh_port);

    let candidates: Vec<u16> = ports.into_iter()
        .filter(|p| !skip.contains(p))
        .collect();

    if candidates.is_empty() {
        return vec![];
    }

    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut handles = Vec::with_capacity(candidates.len());

    for port in candidates {
        let sem = sem.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.ok()?;
            probe(port).await
        }));
    }

    let mut services = Vec::new();
    for handle in handles {
        if let Ok(Some(svc)) = handle.await {
            services.push(svc);
        }
    }
    services.sort_by_key(|s| s.port);
    services
}

// ---------------------------------------------------------------------------
// Port enumeration
// ---------------------------------------------------------------------------

fn listen_ports_linux() -> Vec<u16> {
    let mut ports = HashSet::new();
    for path in &["/proc/net/tcp", "/proc/net/tcp6"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines().skip(1) {
                let mut cols = line.split_ascii_whitespace();
                let _slot   = cols.next();
                let local   = cols.next().unwrap_or("");
                let _remote = cols.next();
                let state   = cols.next().unwrap_or("");
                if state != "0A" { continue; } // 0A = TCP_LISTEN
                if let Some(port_hex) = local.split(':').nth(1) {
                    if let Ok(port) = u16::from_str_radix(port_hex, 16) {
                        if port != 0 {
                            ports.insert(port);
                        }
                    }
                }
            }
        }
    }
    ports.into_iter().collect()
}

fn listen_ports_freebsd() -> Vec<u16> {
    // sockstat -4 -6 -l: USER COMMAND PID FD PROTO LOCAL_ADDR FOREIGN_ADDR
    let output = match std::process::Command::new("sockstat")
        .args(["-4", "-6", "-l"])
        .output()
    {
        Ok(o)  => o,
        Err(_) => return vec![],
    };

    let mut ports = HashSet::new();
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split_ascii_whitespace().collect();
        if cols.len() < 6 { continue; }
        if !cols[4].starts_with("tcp") { continue; }
        if let Some(port_str) = cols[5].rsplit(':').next() {
            if let Ok(port) = port_str.parse::<u16>() {
                if port != 0 {
                    ports.insert(port);
                }
            }
        }
    }
    ports.into_iter().collect()
}

// ---------------------------------------------------------------------------
// HTTP / HTTPS probing
// ---------------------------------------------------------------------------

async fn probe(port: u16) -> Option<HttpService> {
    if let Some(svc) = probe_http(port).await  { return Some(svc); }
    probe_https(port).await
}

async fn probe_http(port: u16) -> Option<HttpService> {
    let tcp = tokio::time::timeout(
        PROBE_TIMEOUT,
        TcpStream::connect(("127.0.0.1", port)),
    )
    .await.ok()?.ok()?;

    send_and_parse(tcp, port, "http").await
}

async fn probe_https(port: u16) -> Option<HttpService> {
    use tokio_rustls::TlsConnector;
    use rustls_pki_types::ServerName;

    let tcp = tokio::time::timeout(
        PROBE_TIMEOUT,
        TcpStream::connect(("127.0.0.1", port)),
    )
    .await.ok()?.ok()?;

    let cfg = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipVerify))
        .with_no_client_auth();

    let connector   = TlsConnector::from(Arc::new(cfg));
    let server_name = ServerName::try_from("localhost").ok()?.to_owned();

    let tls = tokio::time::timeout(
        PROBE_TIMEOUT,
        connector.connect(server_name, tcp),
    )
    .await.ok()?.ok()?;

    send_and_parse(tls, port, "https").await
}

async fn send_and_parse<S>(mut stream: S, port: u16, scheme: &str) -> Option<HttpService>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let req = format!(
        "GET / HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    );
    tokio::time::timeout(PROBE_TIMEOUT, stream.write_all(req.as_bytes()))
        .await.ok()?.ok()?;

    let mut buf = vec![0u8; 8192];
    let n = tokio::time::timeout(PROBE_TIMEOUT, stream.read(&mut buf))
        .await.ok()?.ok()?;

    let text = std::str::from_utf8(&buf[..n]).ok()?;
    if !text.starts_with("HTTP/") {
        return None;
    }

    Some(HttpService {
        port:   port as u32,
        scheme: scheme.to_string(),
        title:  extract_title(text).unwrap_or_default(),
    })
}

fn extract_title(response: &str) -> Option<String> {
    let (_, body) = response.split_once("\r\n\r\n")
        .or_else(|| response.split_once("\n\n"))?;
    let lower = body.to_lowercase();
    let start = lower.find("<title>")? + 7;
    let end   = start + lower[start..].find("</title>")?;
    let title = body[start..end].trim().to_string();
    if title.is_empty() { None } else { Some(title) }
}

// ---------------------------------------------------------------------------
// Always-skip TLS verifier — only used for localhost service discovery
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct SkipVerify;

impl rustls::client::danger::ServerCertVerifier for SkipVerify {
    fn verify_server_cert(
        &self,
        _: &rustls_pki_types::CertificateDer<'_>,
        _: &[rustls_pki_types::CertificateDer<'_>],
        _: &rustls_pki_types::ServerName<'_>,
        _: &[u8],
        _: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &rustls_pki_types::CertificateDer<'_>,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &rustls_pki_types::CertificateDer<'_>,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::aws_lc_rs::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
