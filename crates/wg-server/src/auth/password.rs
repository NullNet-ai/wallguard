use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params, Version,
};

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("hash: {0}")]
    Hash(argon2::password_hash::Error),
    #[error("params: {0}")]
    Params(argon2::Error),
}

impl From<argon2::password_hash::Error> for PasswordError {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::Hash(e)
    }
}

impl From<argon2::Error> for PasswordError {
    fn from(e: argon2::Error) -> Self {
        Self::Params(e)
    }
}

/// Hash a password using Argon2id with m=64 MiB, t=3, p=4.
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt   = SaltString::generate(&mut OsRng);
    let params = Params::new(65536, 3, 4, None)?;  // m=64 MiB (in KiB), t=3, p=4
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);
    let hash   = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a password against a stored argon2id hash string.
///
/// Returns `true` if the password matches, `false` if it does not.
/// Returns an error only if the stored hash is malformed.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let hash = hash_password("hunter2").unwrap();
        assert!(verify_password("hunter2", &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn different_passwords_produce_different_hashes() {
        let h1 = hash_password("abc").unwrap();
        let h2 = hash_password("abc").unwrap();
        // Same password, different salts → different hashes.
        assert_ne!(h1, h2);
    }

    #[test]
    fn malformed_hash_returns_error() {
        assert!(verify_password("x", "not_a_valid_hash").is_err());
    }
}
