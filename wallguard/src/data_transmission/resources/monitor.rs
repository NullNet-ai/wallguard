use async_channel::Receiver;
use std::collections::HashMap;
use std::path::Path;
use sysinfo::{
    Components, CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System,
};

static SYSTEM_REFRESH_KIND: std::sync::LazyLock<RefreshKind> = std::sync::LazyLock::new(|| {
    RefreshKind::nothing()
        .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
        .with_memory(MemoryRefreshKind::nothing().with_ram())
});

static DISK_REFRESH_KIND: std::sync::LazyLock<DiskRefreshKind> =
    std::sync::LazyLock::new(|| DiskRefreshKind::nothing().with_io_usage().with_storage());

/// FreeBSD only exposes `dev.cpu.N.temperature` once the vendor-specific sensor
/// driver is attached; pfSense/OPNsense don't load it by default, so temperature
/// readings are silently empty until we do this ourselves. Loading the wrong
/// vendor's module is harmless: it simply fails to attach to the hardware.
#[cfg(target_os = "freebsd")]
fn load_temperature_sensors() {
    for module in ["coretemp", "amdtemp"] {
        let _ = std::process::Command::new("kldload").arg(module).status();
    }
}

#[derive(Default)]
pub(crate) struct SystemResources {
    pub num_cpus: usize,
    pub global_cpu_usage: f32,
    pub cpu_usages: HashMap<String, f32>,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_disk_space: u64,
    pub available_disk_space: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
    pub temperatures: HashMap<String, Option<f32>>,
}

#[must_use]
pub(crate) fn poll_system_resources(interval_msec: u64) -> Receiver<SystemResources> {
    let (tx, rx) = async_channel::bounded(60);

    std::thread::spawn(move || {
        #[cfg(target_os = "freebsd")]
        load_temperature_sensors();

        let mut sys = System::new_with_specifics(*SYSTEM_REFRESH_KIND);
        let mut disks = Disks::new_with_refreshed_list_specifics(*DISK_REFRESH_KIND);
        let mut components = Components::new_with_refreshed_list();
        #[cfg(target_os = "freebsd")]
        let mut root_disk_io = super::freebsd_disk_io::RootDiskIo::new();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(interval_msec));

            sys.refresh_specifics(*SYSTEM_REFRESH_KIND);
            disks.refresh_specifics(true, *DISK_REFRESH_KIND);
            components.refresh(true);

            let mut cpu_usages = HashMap::new();
            for cpu in sys.cpus() {
                let usage = cpu.cpu_usage();
                cpu_usages.insert(cpu.name().to_string(), usage);
            }

            let mut total_disk_space = 0;
            let mut available_disk_space = 0;
            let mut read_bytes = 0;
            let mut written_bytes = 0;
            for disk in &disks {
                if disk.mount_point() == Path::new("/") {
                    total_disk_space = disk.total_space();
                    available_disk_space = disk.available_space();
                    let disk_usage = disk.usage();
                    read_bytes = disk_usage.read_bytes;
                    written_bytes = disk_usage.written_bytes;
                }
            }

            // On FreeBSD, sysinfo can't attribute I/O to a ZFS-backed root (its
            // dataset name never matches a `/dev/...` device path), so fall back
            // to reading GEOM's devstat counters for the pool's backing disks.
            #[cfg(target_os = "freebsd")]
            if let Some((rb, wb)) = root_disk_io.refresh() {
                read_bytes = rb;
                written_bytes = wb;
            }

            let mut temperatures = HashMap::new();
            for component in &components {
                let temperature = component.temperature();
                temperatures.insert(component.label().to_string(), temperature);
            }

            let resources = SystemResources {
                num_cpus: sys.cpus().len(),
                global_cpu_usage: sys.global_cpu_usage(),
                cpu_usages,
                total_memory: sys.total_memory(),
                used_memory: sys.used_memory(),
                total_disk_space,
                available_disk_space,
                read_bytes,
                written_bytes,
                temperatures,
            };

            // send resources to caller, or exit if channel is closed
            let Ok(()) = tx.send_blocking(resources) else {
                return;
            };
        }
    });

    rx
}
