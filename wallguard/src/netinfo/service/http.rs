use crate::netinfo::service::ServiceInfo;
use crate::netinfo::sock::SocketInfo;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use wallguard_common::cert_verifier::AcceptAllVerifier;

const TIMEOUT_VALUE: Duration = Duration::from_millis(200);

fn create_http_request(addr: SocketAddr) -> String {
    format!(
        "HEAD / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        addr.ip()
    )
}

async fn send_and_check_http_response(
    stream: &mut (impl AsyncReadExt + AsyncWriteExt + Unpin),
    request: &str,
) -> bool {
    if stream.write_all(request.as_bytes()).await.is_err() {
        return false;
    }

    let mut buf = [0u8; 64];
    match stream.read(&mut buf).await {
        Ok(n) if n > 0 => String::from_utf8_lossy(&buf[..n]).starts_with("HTTP/"),
        _ => false,
    }
}

async fn is_http_impl(addr: SocketAddr) -> bool {
    let Ok(mut stream) = TcpStream::connect(addr).await else {
        return false;
    };

    let request = create_http_request(addr);
    send_and_check_http_response(&mut stream, &request).await
}

async fn is_http(addr: SocketAddr) -> bool {
    timeout(TIMEOUT_VALUE, is_http_impl(addr))
        .await
        .unwrap_or(false)
}

fn create_tls_connector() -> TlsConnector {
    let mut config = ClientConfig::builder()
        .with_root_certificates(RootCertStore::empty())
        .with_no_client_auth();

    config
        .dangerous()
        .set_certificate_verifier(Arc::new(AcceptAllVerifier));

    TlsConnector::from(Arc::new(config))
}

async fn is_https_impl(addr: SocketAddr) -> bool {
    let Ok(stream) = TcpStream::connect(addr).await else {
        return false;
    };

    let connector = create_tls_connector();
    let Ok(mut tls_stream) = connector.connect(ServerName::from(addr.ip()), stream).await else {
        return false;
    };

    let request = create_http_request(addr);
    send_and_check_http_response(&mut tls_stream, &request).await
}

async fn is_https(addr: SocketAddr) -> bool {
    timeout(TIMEOUT_VALUE, is_https_impl(addr))
        .await
        .unwrap_or(false)
}

async fn detect_protocol(addr: SocketAddr) -> Option<crate::netinfo::service::Protocol> {
    if is_http(addr).await {
        Some(crate::netinfo::service::Protocol::Http)
    } else if is_https(addr).await {
        Some(crate::netinfo::service::Protocol::Https)
    } else {
        None
    }
}

pub(super) async fn filter(sockets: &[SocketInfo]) -> Vec<ServiceInfo> {
    let tcp_sockets = sockets
        .iter()
        .filter(|s| matches!(s.protocol, crate::netinfo::sock::Protocol::Tcp));

    let mut services = Vec::new();

    for socket in tcp_sockets {
        if let Some(protocol) = detect_protocol(socket.sockaddr).await {
            services.push(ServiceInfo {
                addr: socket.sockaddr,
                protocol,
                program: socket.process_name.clone(),
            });
        }
    }

    services
}
