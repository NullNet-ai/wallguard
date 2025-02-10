use std::path::Path;

use crate::logger::Logger;

#[cfg(debug_assertions)]
pub const BATCH_SIZE: usize = 100;
#[cfg(not(debug_assertions))]
pub const BATCH_SIZE: usize = 10_000;

#[cfg(debug_assertions)]
pub const QUEUE_SIZE: usize = 1_000;
#[cfg(not(debug_assertions))]
pub const QUEUE_SIZE: usize = 1_000_000;
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// Isn't it too long ?
// Full queue can take dozens of GB of RAM

pub static UUID: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| uuid::Uuid::new_v4().to_string());

pub const DUMP_DIR: &str = "packet_dumps";

pub static DISK_SIZE: once_cell::sync::Lazy<u64> = once_cell::sync::Lazy::new(|| {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in &disks {
        if disk.mount_point() == Path::new("/") {
            let available_space = disk.available_space();
            Logger::log(
                log::Level::Info,
                format!("Available disk space: {available_space}"),
            );
            return available_space;
        }
    }
    Logger::log(
        log::Level::Warn,
        "Failed to get disk space, defaulting to 1 GB",
    );
    1_000_000_000
});
