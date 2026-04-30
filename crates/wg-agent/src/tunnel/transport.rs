use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tokio_rustls::TlsConnector;
use tracing::warn;

use super::{TunnelContext, TunnelStream, write_hello};

const QUIC_TIMEOUT: Duration = Duration::from_secs(3);

/// Open a stream to the server's tunnel port, sending the hello header.
///
/// Attempts QUIC first (reusing any open connection); on failure sets the
/// quic_failed flag and falls back to TCP-TLS for the remainder of the session.
pub async fn open_stream(ctx: &TunnelContext, tunnel_id: &str) -> anyhow::Result<TunnelStream> {
    if !ctx.quic_failed.load(Ordering::Relaxed) {
        match try_quic_stream(ctx, tunnel_id).await {
            Ok(s)  => return Ok(s),
            Err(e) => {
                warn!("QUIC tunnel failed ({e:#}); falling back to TCP-TLS");
                ctx.quic_failed.store(true, Ordering::Relaxed);
                *ctx.quic_conn.lock().await = None;
            }
        }
    }
    open_tcp_stream(ctx, tunnel_id).await
}

// ---------------------------------------------------------------------------
// QUIC path
// ---------------------------------------------------------------------------

async fn try_quic_stream(ctx: &TunnelContext, tunnel_id: &str) -> anyhow::Result<TunnelStream> {
    let conn = get_or_create_quic_conn(ctx).await?;

    let (mut send, recv) = tokio::time::timeout(QUIC_TIMEOUT, conn.open_bi())
        .await
        .map_err(|_| anyhow::anyhow!("open_bi timeout"))?
        .map_err(|e| anyhow::anyhow!("open_bi: {e}"))?;

    write_hello(&mut send, tunnel_id).await?;

    Ok(TunnelStream {
        write: Box::new(send),
        read:  Box::new(recv),
    })
}

async fn get_or_create_quic_conn(ctx: &TunnelContext) -> anyhow::Result<quinn::Connection> {
    // Fast path: reuse existing open connection (lock held only briefly).
    let maybe = ctx.quic_conn.lock().await.clone();
    if let Some(conn) = maybe {
        return Ok(conn);
    }

    // Slow path: open a new QUIC connection.
    let conn = tokio::time::timeout(QUIC_TIMEOUT, quic_connect(&ctx.config))
        .await
        .map_err(|_| anyhow::anyhow!("QUIC connect timeout"))?
        .map_err(|e| anyhow::anyhow!("QUIC connect: {e}"))?;

    *ctx.quic_conn.lock().await = Some(conn.clone());
    Ok(conn)
}

async fn quic_connect(config: &crate::config::Config) -> anyhow::Result<quinn::Connection> {
    use quinn::crypto::rustls::QuicClientConfig;

    let tls_cfg  = crate::tls::build_rustls_client_config(config)?;
    let quic_cfg = QuicClientConfig::try_from(tls_cfg)
        .map_err(|e| anyhow::anyhow!("QUIC crypto: {e}"))?;

    let mut client_cfg = quinn::ClientConfig::new(Arc::new(quic_cfg));
    let mut transport  = quinn::TransportConfig::default();
    transport.keep_alive_interval(Some(Duration::from_secs(15)));
    client_cfg.transport_config(Arc::new(transport));

    let addr = tokio::net::lookup_host(
        format!("{}:{}", config.server.name, config.server.quic_port),
    )
    .await?
    .next()
    .ok_or_else(|| anyhow::anyhow!("DNS lookup failed for {}", config.server.name))?;

    let bind_addr: std::net::SocketAddr = if addr.is_ipv6() {
        "[::]:0".parse().unwrap()
    } else {
        "0.0.0.0:0".parse().unwrap()
    };

    let mut endpoint = quinn::Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_cfg);

    let conn = endpoint
        .connect(addr, &config.server.name)
        .map_err(|e| anyhow::anyhow!("QUIC connect initiation: {e}"))?
        .await
        .map_err(|e| anyhow::anyhow!("QUIC handshake: {e}"))?;

    Ok(conn)
}

// ---------------------------------------------------------------------------
// TCP-TLS path
// ---------------------------------------------------------------------------

async fn open_tcp_stream(ctx: &TunnelContext, tunnel_id: &str) -> anyhow::Result<TunnelStream> {
    let config = &ctx.config;

    let addr = tokio::net::lookup_host(
        format!("{}:{}", config.server.name, config.server.tcp_port),
    )
    .await?
    .next()
    .ok_or_else(|| anyhow::anyhow!("DNS lookup failed for {}", config.server.name))?;

    let tcp = tokio::net::TcpStream::connect(addr).await?;

    let tls_cfg = crate::tls::build_rustls_client_config(config)?;
    let connector = TlsConnector::from(Arc::new(tls_cfg));

    let server_name = rustls_pki_types::ServerName::try_from(config.server.name.as_str())
        .map_err(|e| anyhow::anyhow!("invalid server name: {e}"))?
        .to_owned();

    let mut tls = connector.connect(server_name, tcp).await?;

    // Write hello on the full stream before splitting into halves.
    write_hello(&mut tls, tunnel_id).await?;

    let (r, w) = tokio::io::split(tls);
    Ok(TunnelStream {
        write: Box::new(w),
        read:  Box::new(r),
    })
}

