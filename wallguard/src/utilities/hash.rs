use sha2::Digest;
use sha2::Sha256;

pub fn sha256_digest_bytes(input: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.as_slice().try_into().unwrap()
}

pub fn sha256_digest_hex(input: &str) -> String {
    let hash_bytes = sha256_digest_bytes(input);
    hash_bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
