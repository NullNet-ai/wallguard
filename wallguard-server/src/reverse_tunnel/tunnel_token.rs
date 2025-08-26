use nullnet_liberror::{Error, ErrorHandler, Location, location};

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
