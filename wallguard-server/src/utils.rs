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

pub static ACCOUNT_ID: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    std::env::var("ACCOUNT_ID").unwrap_or_else(|_| {
        log::warn!("ACCOUNT_ID environment variable not set");
        String::new()
    })
});

pub static ACCOUNT_SECRET: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    std::env::var("ACCOUNT_SECRET").unwrap_or_else(|_| {
        log::warn!("ACCOUNT_SECRET environment variable not set");
        String::new()
    })
});
