use tokio::net::TcpStream;

use super::{TunnelStream, relay};

/// Relay the tunnel stream to/from the local SSH daemon on `ssh_port`.
pub async fn run_ssh_tunnel(stream: TunnelStream, ssh_port: u16) -> anyhow::Result<()> {
    let tcp = TcpStream::connect(format!("127.0.0.1:{ssh_port}"))
        .await
        .map_err(|e| anyhow::anyhow!("cannot reach SSH daemon on port {ssh_port}: {e}"))?;

    let (r, w) = tcp.into_split();
    relay(stream.read, stream.write, Box::new(r), Box::new(w)).await;
    Ok(())
}
