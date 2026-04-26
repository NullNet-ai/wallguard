use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncReadExt as _;
use tokio_rustls::TlsAcceptor;
use tracing::warn;

use crate::tunnel::{TunnelRegistry, TunnelStream};

/// Spawn both the QUIC (mTLS) and TCP-TLS tunnel listeners.
/// Neither returns; errors during setup are logged and the listener silently
/// does not start (rather than taking down the whole server process).
pub fn spawn_listeners(
    registry:    TunnelRegistry,
    ca_cert_pem: String,
    server_cert: String,
    server_key:  String,
    quic_port:   u16,
    tcp_port:    u16,
) {
    {
        let reg  = registry.clone();
        let ca   = ca_cert_pem.clone();
        let cert = server_cert.clone();
        let key  = server_key.clone();
        tokio::spawn(async move {
            match build_quic_endpoint(&ca, &cert, &key, quic_port) {
                Ok(ep)  => run_quic_listener(ep, reg).await,
                Err(e)  => warn!("QUIC tunnel endpoint setup failed: {e}"),
            }
        });
    }
    {
        let reg  = registry;
        let ca   = ca_cert_pem;
        let cert = server_cert;
        let key  = server_key;
        tokio::spawn(async move {
            match build_tls_acceptor(&ca, &cert, &key) {
                Ok(acceptor) => run_tcp_listener(acceptor, reg, tcp_port).await,
                Err(e)       => warn!("TCP-TLS tunnel acceptor setup failed: {e}"),
            }
        });
    }
}

// ---------------------------------------------------------------------------
// QUIC listener
// ---------------------------------------------------------------------------

fn build_quic_endpoint(
    ca_cert_pem: &str,
    server_cert: &str,
    server_key:  &str,
    port:        u16,
) -> anyhow::Result<quinn::Endpoint> {
    use rustls::server::WebPkiClientVerifier;
    use rustls_pki_types::{CertificateDer, PrivateKeyDer};
    use quinn::crypto::rustls::QuicServerConfig;

    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(
        &mut std::io::Cursor::new(server_cert.as_bytes()),
    ).collect::<Result<Vec<_>, _>>()?;

    let key: PrivateKeyDer<'static> = rustls_pemfile::private_key(
        &mut std::io::Cursor::new(server_key.as_bytes()),
    )?.ok_or_else(|| anyhow::anyhow!("no private key in server PEM"))?;

    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut std::io::Cursor::new(ca_cert_pem.as_bytes())) {
        root_store.add(cert?)?;
    }

    let verifier = WebPkiClientVerifier::builder(Arc::new(root_store))
        .build()
        .map_err(|e| anyhow::anyhow!("client verifier: {e}"))?;

    let tls_cfg = rustls::ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(certs, key)
        .map_err(|e| anyhow::anyhow!("TLS config: {e}"))?;

    let quic_cfg = QuicServerConfig::try_from(tls_cfg)
        .map_err(|e| anyhow::anyhow!("QUIC crypto config: {e}"))?;

    let mut server_cfg = quinn::ServerConfig::with_crypto(Arc::new(quic_cfg));
    let mut transport  = quinn::TransportConfig::default();
    transport.max_concurrent_bidi_streams(quinn::VarInt::from(64u32));
    transport.keep_alive_interval(Some(Duration::from_secs(15)));
    server_cfg.transport_config(Arc::new(transport));

    let addr: std::net::SocketAddr = format!("[::]:{port}").parse()?;
    let endpoint = quinn::Endpoint::server(server_cfg, addr)?;
    Ok(endpoint)
}

async fn run_quic_listener(endpoint: quinn::Endpoint, registry: TunnelRegistry) {
    tracing::info!(addr = ?endpoint.local_addr(), "QUIC tunnel listener started");
    while let Some(incoming) = endpoint.accept().await {
        let reg = registry.clone();
        tokio::spawn(async move {
            match incoming.await {
                Ok(conn) => run_quic_connection(conn, reg).await,
                Err(e)   => warn!("QUIC connection setup error: {e}"),
            }
        });
    }
}

async fn run_quic_connection(conn: quinn::Connection, registry: TunnelRegistry) {
    loop {
        match conn.accept_bi().await {
            Ok((send, recv)) => {
                let reg = registry.clone();
                tokio::spawn(async move {
                    if let Err(e) = dispatch_quic_stream(send, recv, &reg).await {
                        warn!("QUIC tunnel stream error: {e}");
                    }
                });
            }
            Err(e) => {
                tracing::debug!("QUIC connection closed: {e}");
                break;
            }
        }
    }
}

async fn dispatch_quic_stream(
    send:     quinn::SendStream,
    mut recv: quinn::RecvStream,
    registry: &TunnelRegistry,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 36];
    tokio::time::timeout(Duration::from_secs(5), recv.read_exact(&mut buf))
        .await
        .map_err(|_| anyhow::anyhow!("TunnelHello timeout"))?
        .map_err(|e| anyhow::anyhow!("TunnelHello read: {e}"))?;

    let tunnel_id = std::str::from_utf8(&buf)
        .map_err(|e| anyhow::anyhow!("TunnelHello UTF-8: {e}"))?
        .trim()
        .to_string();

    let stream = TunnelStream {
        write: Box::new(send),
        read:  Box::new(recv),
    };

    if !registry.claim(&tunnel_id, stream).await {
        warn!(%tunnel_id, "QUIC: no waiter for tunnel_id — dropping");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// TCP-TLS listener
// ---------------------------------------------------------------------------

fn build_tls_acceptor(
    ca_cert_pem: &str,
    server_cert: &str,
    server_key:  &str,
) -> anyhow::Result<TlsAcceptor> {
    use rustls::server::WebPkiClientVerifier;
    use rustls_pki_types::{CertificateDer, PrivateKeyDer};

    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(
        &mut std::io::Cursor::new(server_cert.as_bytes()),
    ).collect::<Result<Vec<_>, _>>()?;

    let key: PrivateKeyDer<'static> = rustls_pemfile::private_key(
        &mut std::io::Cursor::new(server_key.as_bytes()),
    )?.ok_or_else(|| anyhow::anyhow!("no private key in server PEM"))?;

    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut std::io::Cursor::new(ca_cert_pem.as_bytes())) {
        root_store.add(cert?)?;
    }

    let verifier = WebPkiClientVerifier::builder(Arc::new(root_store))
        .build()
        .map_err(|e| anyhow::anyhow!("client verifier: {e}"))?;

    let tls_cfg = rustls::ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(certs, key)
        .map_err(|e| anyhow::anyhow!("TLS config: {e}"))?;

    Ok(TlsAcceptor::from(Arc::new(tls_cfg)))
}

async fn run_tcp_listener(acceptor: TlsAcceptor, registry: TunnelRegistry, port: u16) {
    let listener = match tokio::net::TcpListener::bind(format!("[::]:{port}")).await {
        Ok(l)  => l,
        Err(e) => { warn!("TCP-TLS tunnel bind failed on port {port}: {e}"); return; }
    };
    tracing::info!(port, "TCP-TLS tunnel listener started");

    loop {
        let (tcp, peer) = match listener.accept().await {
            Ok(a)  => a,
            Err(e) => { warn!("TCP tunnel accept error: {e}"); continue; }
        };
        let acceptor = acceptor.clone();
        let reg      = registry.clone();
        tokio::spawn(async move {
            match acceptor.accept(tcp).await {
                Ok(tls) => {
                    if let Err(e) = dispatch_tls_stream(tls, &reg).await {
                        warn!(%peer, "TCP-TLS tunnel stream error: {e}");
                    }
                }
                Err(e) => warn!(%peer, "TCP-TLS handshake failed: {e}"),
            }
        });
    }
}

async fn dispatch_tls_stream(
    mut stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    registry:   &TunnelRegistry,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 36];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut buf))
        .await
        .map_err(|_| anyhow::anyhow!("TunnelHello timeout"))?
        .map_err(|e| anyhow::anyhow!("TunnelHello read: {e}"))?;

    let tunnel_id = std::str::from_utf8(&buf)
        .map_err(|e| anyhow::anyhow!("TunnelHello UTF-8: {e}"))?
        .trim()
        .to_string();

    let (r, w) = tokio::io::split(stream);
    let ts = TunnelStream {
        write: Box::new(w),
        read:  Box::new(r),
    };

    if !registry.claim(&tunnel_id, ts).await {
        warn!(%tunnel_id, "TCP-TLS: no waiter for tunnel_id — dropping");
    }
    Ok(())
}
