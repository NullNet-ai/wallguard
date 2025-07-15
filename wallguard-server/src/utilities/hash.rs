use sha2::Digest;
use sha2::Sha256;

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

/// Computes the MD5 hash of a given string.
///
/// # Arguments
/// * `input` - A string slice (`&str`) to be hashed.
///
/// # Returns
/// A `String` representing the MD5 hash in hexadecimal format.
pub fn md5_digest(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}
