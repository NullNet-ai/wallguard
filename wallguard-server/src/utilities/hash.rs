use sha2::Digest;
use sha2::Sha256;

use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher};

/// Calculates the SHA-256 digest of the input string and returns it as a 32-byte array.
///
/// # Arguments
/// * `input` - The input string to hash.
///
/// # Returns
/// A 32-byte SHA-256 digest.
pub fn sha256_digest_bytes(input: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.as_slice().try_into().unwrap()
}

/// Hashes the given secret using the Argon2 algorithm with a randomly generated salt.
///
/// # Arguments
/// * `secret` - The plaintext secret to be hashed (e.g., a password).
///
/// # Returns
/// * `Ok(String)` - The resulting Argon2 hash as a string in PHC format, including parameters and salt.
/// * `Err(String)` - An error message if hashing fails (e.g., due to internal errors or resource limits).
pub fn hash_secret(secret: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(secret.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(err) => Err(format!("Hashing failed: {}", err)),
    }
}
