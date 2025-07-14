use crate::utilities::hash::sha256_digest_bytes;
use crate::utilities::random::generate_random_string;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::{io::AsyncReadExt, net::TcpStream};

/// The expected size (in bytes) of a SHA-256 token hash.
pub const TOKEN_HASH_SIZE: usize = 32;

/// A fixed-size wrapper around a SHA-256 digest used to identify a tunnel connection.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct TokenHash {
    digest: [u8; TOKEN_HASH_SIZE],
}

impl TokenHash {
    /// Reads a 32-byte token hash from the beginning of a TCP stream.
    ///
    /// This function assumes that the first message received on the stream
    /// is a fixed-size SHA-256 hash that can be used to identify the reverse tunnel.
    ///
    /// # Errors
    /// Returns an error if reading from the stream fails or fewer than 32 bytes are received.
    pub async fn read_from_stream(stream: &mut TcpStream) -> Result<Self, Error> {
        let mut hash = TokenHash::default();

        stream
            .read_exact(&mut hash.digest)
            .await
            .handle_err(location!())?;

        Ok(hash)
    }
}

impl From<[u8; TOKEN_HASH_SIZE]> for TokenHash {
    fn from(digest: [u8; TOKEN_HASH_SIZE]) -> Self {
        Self { digest }
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
