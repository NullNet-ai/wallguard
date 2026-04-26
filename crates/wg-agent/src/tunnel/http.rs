use tokio::net::TcpStream;

use super::{TunnelStream, relay};

/// Relay the tunnel stream to/from an arbitrary TCP target (HTTP proxy tunnel).
pub async fn run_http_tunnel(
    stream:      TunnelStream,
    target_host: &str,
    target_port: u16,
) -> anyhow::Result<()> {
    let tcp = TcpStream::connect(format!("{target_host}:{target_port}"))
        .await
        .map_err(|e| anyhow::anyhow!("cannot connect to {target_host}:{target_port}: {e}"))?;

    let (r, w) = tcp.into_split();
    relay(stream.read, stream.write, Box::new(r), Box::new(w)).await;
    Ok(())
}
