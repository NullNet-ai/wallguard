/// Computes the MD5 hash of a given string.
///
/// # Arguments
/// * `input` - A string slice (`&str`) to be hashed.
///
/// # Returns
/// A `String` representing the MD5 hash in hexadecimal format.
pub fn digest(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}
