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

pub static ACCOUNT_ID: once_cell::sync::Lazy<&str> = once_cell::sync::Lazy::new(|| {
    option_env!("ACCOUNT_ID").unwrap_or({
        log::warn!("ACCOUNT_ID environment variable not set");
        ""
    })
});

pub static ACCOUNT_SECRET: once_cell::sync::Lazy<&str> = once_cell::sync::Lazy::new(|| {
    option_env!("ACCOUNT_SECRET").unwrap_or({
        log::warn!("ACCOUNT_SECRET environment variable not set");
        ""
    })
});
