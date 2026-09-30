use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::fmt::{Display, Formatter};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::utilities::hash::sha256_digest_bytes;
use crate::utilities::random::generate_random_string;

/// The expected size (in bytes) of a SHA-256 token hash.
pub const TOKEN_HASH_SIZE: usize = 32;

/// A fixed-size wrapper around a SHA-256 digest used to identify a tunnel connection.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct TokenHash {
    digest: [u8; TOKEN_HASH_SIZE],
}

impl TryFrom<Vec<u8>> for TokenHash {
    type Error = Error;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        let digest: [u8; TOKEN_HASH_SIZE] = vec
            .try_into()
            .map_err(|_| "Expected a token hash of exact length 32 bytes")
            .handle_err(location!())?;
        Ok(TokenHash { digest })
    }
}

impl From<[u8; TOKEN_HASH_SIZE]> for TokenHash {
    fn from(digest: [u8; TOKEN_HASH_SIZE]) -> Self {
        Self { digest }
    }
}

impl TokenHash {
    /// Reads a 32-byte token hash from the beginning of a TCP stream.
    ///
    /// This function assumes that the first message received on the stream
    /// is a fixed-size SHA-256 hash that can be used to identify the reverse tunnel.
    ///
    /// # Errors
    /// Returns a [`HandshakeError`] if the stream fails, closes before 32 bytes
    /// are received, or `timeout` elapses. The error carries whatever bytes did
    /// arrive, which helps identify what is connecting to the port.
    pub async fn read_from_stream(
        stream: &mut TcpStream,
        timeout: Duration,
    ) -> Result<Self, HandshakeError> {
        let mut hash = TokenHash::default();
        let mut received = 0;

        let result = tokio::time::timeout(timeout, async {
            while received < TOKEN_HASH_SIZE {
                match stream.read(&mut hash.digest[received..]).await {
                    Ok(0) => return Err(HandshakeFailure::Closed),
                    Ok(n) => received += n,
                    Err(err) => return Err(HandshakeFailure::Io(err)),
                }
            }
            Ok(())
        })
        .await;

        let failure = match result {
            Ok(Ok(())) => return Ok(hash),
            Ok(Err(failure)) => failure,
            Err(_) => HandshakeFailure::Timeout(timeout),
        };

        Err(HandshakeError {
            failure,
            received: hash.digest[..received].to_vec(),
        })
    }
}

#[derive(Debug)]
pub enum HandshakeFailure {
    Closed,
    Timeout(Duration),
    Io(std::io::Error),
}

/// Failure to read a token hash, along with the bytes received before it.
#[derive(Debug)]
pub struct HandshakeError {
    pub failure: HandshakeFailure,
    pub received: Vec<u8>,
}

impl Display for HandshakeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.failure {
            HandshakeFailure::Closed => write!(f, "peer closed the connection")?,
            HandshakeFailure::Timeout(timeout) => write!(f, "timed out after {timeout:?}")?,
            HandshakeFailure::Io(err) => write!(f, "I/O error: {err}")?,
        }

        write!(f, " after {}/{TOKEN_HASH_SIZE} bytes", self.received.len())?;

        if !self.received.is_empty() {
            // Printable preview makes common probes obvious, e.g. "GET / HTTP"
            // from an HTTP health check or 0x16 0x03 from a TLS ClientHello.
            let hex: String = self.received.iter().map(|b| format!("{b:02x}")).collect();
            let text: String = self
                .received
                .iter()
                .map(|&b| {
                    if b.is_ascii_graphic() || b == b' ' {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            write!(f, " (hex: {hex}, text: {text:?})")?;
        }

        Ok(())
    }
}

/// Represents a randomly generated authentication token for reverse tunnels.
///
/// This token is not transmitted directly—instead, a SHA-256 hash of the token is sent
/// for authentication purposes. This avoids the overhead of parsing variable-length strings
/// and enables fixed-size, efficient, and predictable connection handshakes.
#[derive(Debug, Clone)]
pub struct TunnelToken {
    token: String,
}

impl TunnelToken {
    /// Generates a new random alphanumeric token.
    /// The corresponding hash will later be used to authenticate a tunnel.
    pub fn generate() -> Self {
        let token = generate_random_string(32);
        Self { token }
    }
}

impl From<TunnelToken> for String {
    fn from(value: TunnelToken) -> Self {
        value.token
    }
}

impl From<TunnelToken> for TokenHash {
    fn from(value: TunnelToken) -> Self {
        sha256_digest_bytes(&value.token).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    async fn handshake(
        payload: &'static [u8],
        keep_open: bool,
    ) -> Result<TokenHash, HandshakeError> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream.write_all(payload).await.unwrap();
            if keep_open {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        let (mut stream, _) = listener.accept().await.unwrap();
        let result = TokenHash::read_from_stream(&mut stream, Duration::from_millis(200)).await;
        client.await.unwrap();
        result
    }

    #[tokio::test]
    async fn reads_full_hash() {
        let hash = handshake(&[7u8; TOKEN_HASH_SIZE], false).await.unwrap();
        assert_eq!(hash, TokenHash::from([7u8; TOKEN_HASH_SIZE]));
    }

    #[tokio::test]
    async fn reports_bytes_received_before_close() {
        let err = handshake(b"GET / HTTP/1.1\r\n", false).await.unwrap_err();
        assert!(matches!(err.failure, HandshakeFailure::Closed));
        assert_eq!(err.received, b"GET / HTTP/1.1\r\n");
        let msg = err.to_string();
        assert!(msg.contains("16/32 bytes"), "{msg}");
        assert!(msg.contains(r#"text: "GET / HTTP/1.1..""#), "{msg}");
    }

    #[tokio::test]
    async fn reports_timeout() {
        let err = handshake(b"ab", true).await.unwrap_err();
        assert!(matches!(err.failure, HandshakeFailure::Timeout(_)));
        assert_eq!(err.received, b"ab");
    }
}
