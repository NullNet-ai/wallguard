use crate::netinfo::service::Protocol;
use crate::netinfo::sock::SocketInfo;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use wallguard_common::cert_verifier::AcceptAllVerifier;

const TIMEOUT_VALUE: Duration = Duration::from_millis(200);

fn create_http_request(target: SocketAddr) -> String {
    let host = match target {
        SocketAddr::V4(v4) => v4.ip().to_string(),
        SocketAddr::V6(v6) => format!("[{}]", v6.ip()),
    };

    format!("HEAD / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n")
}

async fn send_and_check_http_response(
    stream: &mut (impl AsyncReadExt + AsyncWriteExt + Unpin),
    request: &str,
) -> Option<i32> {
    if stream.write_all(request.as_bytes()).await.is_err() {
        return None;
    }

    let mut buf = [0u8; 128];

    let n = match stream.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return None,
    };

    let response = String::from_utf8_lossy(&buf[..n]);

    let mut parts = response.split_whitespace();

    match (parts.next(), parts.next()) {
        (Some(http), Some(code)) if http.starts_with("HTTP/") => code.parse::<i32>().ok(),
        _ => None,
    }
}

async fn is_http_impl(addr: SocketAddr) -> Option<i32> {
    let (mut stream, target) = super::connect_probe(addr).await?;

    let request = create_http_request(target);
    send_and_check_http_response(&mut stream, &request).await
}

async fn is_http(addr: SocketAddr) -> Option<i32> {
    timeout(TIMEOUT_VALUE, is_http_impl(addr))
        .await
        .unwrap_or(None)
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

async fn is_https_impl(addr: SocketAddr) -> Option<i32> {
    let (stream, target) = super::connect_probe(addr).await?;

    let connector = create_tls_connector();
    let Ok(mut tls_stream) = connector
        .connect(ServerName::from(target.ip()), stream)
        .await
    else {
        return None;
    };

    let request = create_http_request(target);
    send_and_check_http_response(&mut tls_stream, &request).await
}

async fn is_https(addr: SocketAddr) -> Option<i32> {
    timeout(TIMEOUT_VALUE, is_https_impl(addr))
        .await
        .unwrap_or(None)
}

async fn detect_protocol(addr: SocketAddr) -> Option<(Protocol, i32)> {
    if let Some(retval) = is_https(addr).await.map(|code| (Protocol::Https, code)) {
        Some(retval)
    } else {
        is_http(addr).await.map(|code| (Protocol::Http, code))
    }
}

pub(super) async fn filter(sockets: &mut Vec<SocketInfo>) -> Vec<(SocketInfo, Protocol)> {
    let mut services = Vec::new();
    let mut remaining = Vec::with_capacity(sockets.len());

    // Probe every candidate socket concurrently instead of awaiting each TLS
    // handshake + HTTP probe one at a time: sequential probing made this scale
    // linearly (up to TIMEOUT_VALUE per socket) with the number of open
    // listening ports every scan cycle.
    let mut set = tokio::task::JoinSet::new();
    for socket in sockets.drain(..) {
        set.spawn(async move {
            let detected = if matches!(socket.protocol, crate::netinfo::sock::Protocol::Tcp) {
                detect_protocol(socket.sockaddr).await
            } else {
                None
            };
            (socket, detected)
        });
    }

    while let Some(joined) = set.join_next().await {
        let Ok((socket, detected)) = joined else {
            continue;
        };

        match detected {
            Some((protocol, code)) if (200..300).contains(&code) => {
                services.push((socket, protocol));
            }
            _ => remaining.push(socket),
        }
    }

    *sockets = remaining;
    services
}
