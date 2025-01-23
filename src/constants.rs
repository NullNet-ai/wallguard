#[cfg(debug_assertions)]
pub const BATCH_SIZE: usize = 100;
#[cfg(not(debug_assertions))]
pub const BATCH_SIZE: usize = 10_000;

#[cfg(debug_assertions)]
pub const QUEUE_SIZE: usize = 1_000;
#[cfg(not(debug_assertions))]
pub const QUEUE_SIZE: usize = 1_000_000;

pub static UUID: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| uuid::Uuid::new_v4().to_string());
