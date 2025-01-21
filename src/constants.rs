#[cfg(debug_assertions)]
pub const BUFFER_SIZE: usize = 128;
#[cfg(not(debug_assertions))]
pub const BUFFER_SIZE: usize = 1024;

pub static UUID: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| uuid::Uuid::new_v4().to_string());
