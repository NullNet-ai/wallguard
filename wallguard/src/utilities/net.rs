use socket2::{SockRef, TcpKeepalive};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

/// Upper bound on the TCP handshake. A bare `TcpStream::connect` to a peer
/// that silently drops SYNs only gives up after the kernel's SYN retries
/// (~2 minutes on Linux), holding the caller hostage the whole time.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Keepalive probing starts after this much silence on the connection...
const KEEPALIVE_TIME: Duration = Duration::from_secs(60);
/// ...and then repeats at this interval until the OS retry limit is hit.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(15);

/// Once one direction of a relay has finished, the other direction is given
/// this long of inactivity to flush whatever is still in flight before both
/// sockets are torn down.
const HALF_CLOSE_IDLE: Duration = Duration::from_secs(10);

const RELAY_BUFFER_SIZE: usize = 16 * 1024;

/// Connects to `addr` with a bounded handshake and TCP keepalive enabled.
///
/// Keepalive is what lets a long-lived tunnel notice that its peer is gone
/// when the connection dies without a FIN/RST (server crash, NAT/LB idle
/// eviction, network partition). Without it a blocked `read` on such a
/// socket never returns and the socket — along with whatever it has queued —
/// is held forever.
pub async fn connect(addr: impl ToSocketAddrs) -> io::Result<TcpStream> {
    let stream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(addr))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "TCP connect timed out"))??;

    enable_keepalive(&stream)?;

    Ok(stream)
}

pub fn enable_keepalive(stream: &TcpStream) -> io::Result<()> {
    let keepalive = TcpKeepalive::new()
        .with_time(KEEPALIVE_TIME)
        .with_interval(KEEPALIVE_INTERVAL);

    SockRef::from(stream).set_tcp_keepalive(&keepalive)
}

/// Addresses to dial, in order, to reach a local listener bound to `addr`.
///
/// A wildcard listener can't be dialed at its own address, so it is
/// reached over loopback. IPv4 loopback comes first even for `[::]`: most `[::]`
/// listeners are dual-stack, and IPv6 loopback can reach a different socket
/// than real clients do. Docker Swarm publishes a service port as a
/// dual-stack socket held by `dockerd` that never accepts, while the
/// routing mesh intercepts IPv4 traffic only, so a connection to `[::1]` ends
/// up queued in that socket forever. `[::1]` is only tried when the IPv4
/// connect fails, which is the case for IPv6-only listeners.
pub fn local_dial_targets(addr: SocketAddr) -> Vec<SocketAddr> {
    let port = addr.port();

    match addr.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => {
            vec![SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port)]
        }
        IpAddr::V6(ip) if ip.is_unspecified() => vec![
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port),
            SocketAddr::new(Ipv6Addr::LOCALHOST.into(), port),
        ],
        _ => vec![addr],
    }
}

/// Connects to the local listener bound to `addr` (see `local_dial_targets`)
/// with a bounded handshake and TCP keepalive.
pub async fn connect_local(addr: SocketAddr) -> io::Result<TcpStream> {
    let mut last_err = None;

    for target in local_dial_targets(addr) {
        match connect(target).await {
            Ok(stream) => return Ok(stream),
            Err(err) => last_err = Some(err),
        }
    }

    Err(last_err.unwrap_or_else(|| io::Error::other("no address to dial")))
}

/// Tracks the last time any byte crossed a relay, in either direction.
struct Activity {
    start: Instant,
    last_ms: AtomicU64,
}

impl Activity {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            last_ms: AtomicU64::new(0),
        }
    }

    fn touch(&self) {
        let now = self.start.elapsed().as_millis() as u64;
        self.last_ms.store(now, Ordering::Relaxed);
    }

    fn idle_for(&self) -> Duration {
        let last = Duration::from_millis(self.last_ms.load(Ordering::Relaxed));
        self.start.elapsed().saturating_sub(last)
    }
}

/// Resolves once nothing has crossed the relay for `timeout`; never
/// resolves when `timeout` is `None`.
async fn wait_idle(activity: &Activity, timeout: Option<Duration>) {
    let Some(timeout) = timeout else {
        return std::future::pending().await;
    };

    loop {
        let idle_for = activity.idle_for();
        if idle_for >= timeout {
            return;
        }
        tokio::time::sleep(timeout - idle_for).await;
    }
}

async fn pipe<R, W>(reader: &mut R, writer: &mut W, activity: &Activity) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buffer = vec![0u8; RELAY_BUFFER_SIZE];

    loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            // Propagate the half-close so the far side sees EOF as well.
            let _ = writer.shutdown().await;
            return Ok(());
        }

        writer.write_all(&buffer[..n]).await?;
        activity.touch();
    }
}

/// Relays bytes between `a` and `b` until the session is over, then drops
/// both.
///
/// `tokio::io::copy_bidirectional` only returns once *both* directions have
/// reached EOF, so a peer that half-closes (or vanishes) leaves the other
/// socket open indefinitely. Here, as soon as one direction finishes the
/// other gets `HALF_CLOSE_IDLE` of inactivity to drain, and the whole relay
/// is abandoned if nothing flows in either direction for `idle_timeout`.
pub async fn relay<A, B>(a: A, b: B, idle_timeout: Option<Duration>)
where
    A: AsyncRead + AsyncWrite,
    B: AsyncRead + AsyncWrite,
{
    let (mut a_reader, mut a_writer) = tokio::io::split(a);
    let (mut b_reader, mut b_writer) = tokio::io::split(b);
    let activity = Activity::new();

    let a_to_b = pipe(&mut a_reader, &mut b_writer, &activity);
    let b_to_a = pipe(&mut b_reader, &mut a_writer, &activity);
    tokio::pin!(a_to_b, b_to_a);

    tokio::select! {
        _ = &mut a_to_b => {
            // Start the drain grace period from the moment of the half-close.
            activity.touch();
            tokio::select! {
                _ = &mut b_to_a => {}
                _ = wait_idle(&activity, Some(HALF_CLOSE_IDLE)) => {}
            }
        }
        _ = &mut b_to_a => {
            activity.touch();
            tokio::select! {
                _ = &mut a_to_b => {}
                _ = wait_idle(&activity, Some(HALF_CLOSE_IDLE)) => {}
            }
        }
        _ = wait_idle(&activity, idle_timeout) => {
            log::debug!("Relay idle for {idle_timeout:?}, closing");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[test]
    fn wildcard_listeners_are_dialed_over_ipv4_loopback_first() {
        let v4_any: SocketAddr = "0.0.0.0:3000".parse().unwrap();
        let v6_any: SocketAddr = "[::]:3000".parse().unwrap();
        let specific: SocketAddr = "192.0.2.7:3000".parse().unwrap();

        assert_eq!(
            local_dial_targets(v4_any),
            vec!["127.0.0.1:3000".parse::<SocketAddr>().unwrap()]
        );
        assert_eq!(
            local_dial_targets(v6_any),
            vec![
                "127.0.0.1:3000".parse::<SocketAddr>().unwrap(),
                "[::1]:3000".parse::<SocketAddr>().unwrap(),
            ]
        );
        assert_eq!(local_dial_targets(specific), vec![specific]);
    }

    #[tokio::test]
    async fn relay_ends_when_one_side_closes_and_other_stays_open() {
        let (a_local, a_remote) = duplex(1024);
        let (b_local, mut b_remote) = duplex(1024);

        let relay = tokio::spawn(relay(a_remote, b_local, None));

        // `a` goes away; `b` stays open and never sends anything, which
        // left `copy_bidirectional` waiting forever.
        drop(a_local);

        tokio::time::timeout(HALF_CLOSE_IDLE + Duration::from_secs(5), relay)
            .await
            .expect("relay should finish after the half-close grace period")
            .unwrap();

        // Both halves of `b` were dropped with the relay.
        let mut buf = [0u8; 1];
        assert_eq!(b_remote.read(&mut buf).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn relay_forwards_data_and_honours_idle_timeout() {
        let (mut a_local, a_remote) = duplex(1024);
        let (b_local, mut b_remote) = duplex(1024);

        let relay = tokio::spawn(relay(a_remote, b_local, Some(Duration::from_millis(300))));

        a_local.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        b_remote.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");

        tokio::time::timeout(Duration::from_secs(5), relay)
            .await
            .expect("idle relay should be torn down")
            .unwrap();
    }
}
