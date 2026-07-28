#![cfg(target_os = "freebsd")]

//! `sysinfo`'s FreeBSD disk-I/O accounting attributes bytes to a mount point by
//! matching its underlying device against a `/dev/...` path resolved from
//! `kern.geom.conftxt`. That works for UFS, but a ZFS-backed mount reports its
//! source as a dataset name (e.g. "zroot/ROOT/default") which never matches a
//! `/dev/...` path, so `read_bytes`/`written_bytes` stay 0 forever on any
//! ZFS-rooted box — the default on modern pfSense/OPNsense installs.
//!
//! This module fills that gap for the root filesystem specifically: it resolves
//! the ZFS pool backing "/" to its physical leaf disks via `zpool status`, then
//! reads GEOM's raw devstat counters for exactly those disks (never summing
//! every GEOM layer, which would double-count the same I/O once per partition
//! and label provider stacked on top of each disk).

use libc::{c_char, c_int, c_void, devstat, devstat_getversion, size_t};
use std::ffi::CString;
use std::process::Command;
use std::ptr::null_mut;
use std::sync::OnceLock;

const DEVSTAT_READ: usize = 0x01;
const DEVSTAT_WRITE: usize = 0x02;

#[link(name = "geom")]
unsafe extern "C" {
    fn geom_stats_open() -> c_int;
    fn geom_stats_snapshot_get() -> *mut c_void;
    fn geom_stats_snapshot_next(arg: *mut c_void) -> *mut devstat;
    fn geom_stats_snapshot_free(arg: *mut c_void);
}

fn geom_ready() -> bool {
    static READY: OnceLock<bool> = OnceLock::new();
    *READY.get_or_init(|| unsafe { devstat_getversion(null_mut()) == 6 && geom_stats_open() == 0 })
}

fn c_buf_to_string(buf: &[c_char]) -> Option<String> {
    let bytes: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast(), buf.len()) };
    let len = bytes.iter().position(|&b| b == 0)?;
    std::str::from_utf8(&bytes[..len]).ok().map(str::to_owned)
}

/// Reads a string-valued sysctl by name (e.g. "kern.geom.conftxt").
fn sysctl_string(name: &str) -> Option<String> {
    let c_name = CString::new(name).ok()?;
    let mut len: size_t = 0;
    unsafe {
        if libc::sysctlbyname(
            c_name.as_ptr(),
            null_mut(),
            &mut len,
            null_mut(),
            0,
        ) != 0
        {
            return None;
        }
        let mut buf = vec![0u8; len];
        if libc::sysctlbyname(
            c_name.as_ptr(),
            buf.as_mut_ptr().cast(),
            &mut len,
            null_mut(),
            0,
        ) != 0
        {
            return None;
        }
        buf.truncate(len);
        // The sysctl is a NUL-terminated C string; trailing NULs would otherwise
        // survive into the String and break later exact-match comparisons.
        while buf.last() == Some(&0) {
            buf.pop();
        }
        String::from_utf8(buf).ok()
    }
}

/// Maps every alternate `/dev/...` path GEOM knows for a disk (partitions, GPT
/// labels, gptid, diskid, ...) back to that disk's base name (e.g. "ada0").
/// Mirrors the mapping `sysinfo` itself builds from the same sysctl internally.
fn disk_label_mapping() -> std::collections::HashMap<String, String> {
    let mut mapping = std::collections::HashMap::new();
    let Some(conftxt) = sysctl_string("kern.geom.conftxt") else {
        return mapping;
    };

    let mut last_id = String::new();
    for line in conftxt.lines() {
        let mut parts = line.split_whitespace();
        let Some(kind) = parts.next() else { continue };

        if kind == "0" {
            if let Some("DISK") = parts.next()
                && let Some(id) = parts.next()
            {
                last_id.clear();
                last_id.push_str(id);
            }
        } else if kind == "2" && !last_id.is_empty() {
            if let Some("LABEL") = parts.next()
                && let Some(path) = parts.next()
            {
                mapping.insert(format!("/dev/{path}"), last_id.clone());
            }
        }
    }
    mapping
}

/// "ada0p3" -> "ada0", "nvd0p2" -> "nvd0", "da1" (whole-disk vdev) -> "da1".
fn strip_partition_suffix(device: &str) -> String {
    match device.rfind('p') {
        Some(idx)
            if idx + 1 < device.len() && device[idx + 1..].bytes().all(|b| b.is_ascii_digit()) =>
        {
            device[..idx].to_string()
        }
        _ => device.to_string(),
    }
}

/// Resolves a `zpool status` leaf device entry (a raw name like "ada0p3", or a
/// label like "gpt/zfs0" / "gptid/<uuid>") to the base GEOM disk name that
/// devstat actually tracks.
fn resolve_base_disk(leaf: &str, label_mapping: &std::collections::HashMap<String, String>) -> Option<String> {
    if leaf.contains('/') {
        label_mapping.get(&format!("/dev/{leaf}")).cloned()
    } else {
        Some(strip_partition_suffix(leaf))
    }
}

/// Runs `zpool status <pool>` and returns the base disk names backing every
/// leaf vdev (mirrors/raidz members included; spares and cache/log devices are
/// included too since they're real devices attached to the pool).
fn zpool_backing_disks(pool: &str) -> Vec<String> {
    let Ok(output) = Command::new("/sbin/zpool").arg("status").arg(pool).output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let label_mapping = disk_label_mapping();

    let mut in_config = false;
    let mut seen_header = false;
    let mut disks = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "config:" {
            in_config = true;
            continue;
        }
        if !in_config {
            continue;
        }
        if trimmed.is_empty() {
            if seen_header {
                break;
            }
            continue;
        }
        let Some(name) = trimmed.split_whitespace().next() else {
            continue;
        };
        if name == "NAME" {
            seen_header = true;
            continue;
        }
        if name == pool
            || name.starts_with("mirror-")
            || name.starts_with("raidz")
            || name == "spares"
            || name == "logs"
            || name == "cache"
        {
            continue;
        }
        if let Some(base) = resolve_base_disk(name, &label_mapping) {
            disks.push(base);
        }
    }

    disks.sort();
    disks.dedup();
    disks
}

/// Sums read/write bytes across exactly the named base-disk devstat entries,
/// never across every GEOM layer (which would multiply-count the same I/O).
fn read_named_disk_totals(names: &[String]) -> Option<(u64, u64)> {
    if !geom_ready() || names.is_empty() {
        return None;
    }
    let snap = unsafe { geom_stats_snapshot_get() };
    if snap.is_null() {
        return None;
    }

    let mut read_bytes = 0u64;
    let mut written_bytes = 0u64;
    loop {
        let device = unsafe { geom_stats_snapshot_next(snap) };
        if device.is_null() {
            break;
        }
        let device = unsafe { &*device };
        let Some(device_name) = c_buf_to_string(&device.device_name) else {
            continue;
        };
        // Avoid allocating a fresh "{device_name}{unit_number}" String for
        // every GEOM provider on the system (there can be dozens once
        // partitions/labels are counted) just to compare it against the
        // handful of names we actually care about.
        let is_named_disk = names.iter().any(|n| {
            n.strip_prefix(device_name.as_str())
                .is_some_and(|suffix| suffix.parse::<c_int>() == Ok(device.unit_number))
        });
        if is_named_disk {
            read_bytes = read_bytes.saturating_add(device.bytes[DEVSTAT_READ]);
            written_bytes = written_bytes.saturating_add(device.bytes[DEVSTAT_WRITE]);
        }
    }
    unsafe { geom_stats_snapshot_free(snap) };
    Some((read_bytes, written_bytes))
}

/// Tracks cumulative-to-delta conversion for the root filesystem's backing
/// disk(s), across polling intervals.
pub(crate) struct RootDiskIo {
    disk_names: Option<Vec<String>>,
    prev: Option<(u64, u64)>,
}

impl RootDiskIo {
    pub(crate) fn new() -> Self {
        let disk_names = mount_source("/").and_then(|(fstype, source)| {
            if fstype != "zfs" {
                // Not ZFS: sysinfo's own dev_id-based matching already works here.
                return None;
            }
            let pool = source.split('/').next().unwrap_or(&source);
            let disks = zpool_backing_disks(pool);
            if disks.is_empty() { None } else { Some(disks) }
        });
        Self {
            disk_names,
            prev: None,
        }
    }

    /// Returns `(read_bytes, written_bytes)` deltas since the last call, or
    /// `None` when "/" isn't ZFS or pool resolution failed — callers should
    /// keep whatever `sysinfo` already computed in that case.
    pub(crate) fn refresh(&mut self) -> Option<(u64, u64)> {
        let names = self.disk_names.as_ref()?;
        let (total_read, total_written) = read_named_disk_totals(names)?;
        let delta = match self.prev {
            Some((prev_read, prev_written)) => (
                total_read.saturating_sub(prev_read),
                total_written.saturating_sub(prev_written),
            ),
            None => (0, 0),
        };
        self.prev = Some((total_read, total_written));
        Some(delta)
    }
}

/// Returns the raw fstype + mount-source ("f_mntfromname") for a path, as
/// reported by `statfs(2)`. For ZFS this is the dataset name; for UFS/other
/// it's a device path.
fn mount_source(path: &str) -> Option<(String, String)> {
    let c_path = CString::new(path).ok()?;
    let mut buf: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statfs(c_path.as_ptr(), &mut buf) } != 0 {
        return None;
    }
    let fstype = c_buf_to_string(&buf.f_fstypename)?;
    let source = c_buf_to_string(&buf.f_mntfromname)?;
    Some((fstype, source))
}
