use std::path::Path;

#[cfg(debug_assertions)]
pub const BATCH_SIZE: usize = 1000;
#[cfg(not(debug_assertions))]
pub const BATCH_SIZE: usize = 10_000;

#[cfg(debug_assertions)]
pub const QUEUE_SIZE: usize = 10_000;
#[cfg(not(debug_assertions))]
pub const QUEUE_SIZE: usize = 1_000_000;

#[cfg(debug_assertions)]
pub const QUEUE_SIZE_RESOURCES: usize = 10;
#[cfg(not(debug_assertions))]
pub const QUEUE_SIZE_RESOURCES: usize = 60;

pub const SNAPLEN: usize = 96;

pub const DUMP_DIR: &str = "dumps";

pub static DISK_SIZE: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in &disks {
        if disk.mount_point() == Path::new("/") {
            let available_space = disk.available_space();
            log::info!("Available disk space: {available_space}");
            return available_space;
        }
    }
    log::warn!("Failed to get disk space, defaulting to 1 GB",);
    1_000_000_000
});

pub const DATA_TRANSMISSION_INTERVAL_SECONDS: u64 = 1;
