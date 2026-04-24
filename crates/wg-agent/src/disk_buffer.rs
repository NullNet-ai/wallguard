use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::warn;

/// Persistent write-ahead buffer for packet batches that could not be
/// uploaded while the server was unreachable.
///
/// Files are named `{id:016x}.bin`; lexicographic order matches insertion
/// order.  All methods are infallible from the caller's perspective:
/// errors are logged and a conservative sentinel is returned.
pub struct DiskBuffer {
    dir:        PathBuf,
    max_bytes:  u64,
    min_free:   u64,
    next_id:    AtomicU64,
    used_bytes: AtomicU64,
}

impl DiskBuffer {
    /// Open (or create) the buffer directory and restore state from any
    /// existing files left over from a previous run.
    pub fn new(dir: PathBuf, max_bytes: u64, min_free_bytes: u64) -> Self {
        let _ = std::fs::create_dir_all(&dir);

        let mut max_id: u64 = 0;
        let mut used:   u64 = 0;

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name();
                let name  = fname.to_string_lossy();
                if let Some(hex) = name.strip_suffix(".bin") {
                    if let Ok(id) = u64::from_str_radix(hex, 16) {
                        max_id = max_id.max(id + 1);
                    }
                }
                used = used.saturating_add(
                    entry.metadata().map(|m| m.len()).unwrap_or(0),
                );
            }
        }

        Self {
            dir,
            max_bytes,
            min_free: min_free_bytes,
            next_id:    AtomicU64::new(max_id),
            used_bytes: AtomicU64::new(used),
        }
    }

    /// Persist `data` to a new buffer file.
    ///
    /// Returns `false` without writing if:
    /// - the total buffer size would exceed `max_bytes`, or
    /// - the filesystem has less than `min_free_bytes` available.
    pub fn try_write(&self, data: &[u8]) -> bool {
        let size = data.len() as u64;

        if self.used_bytes.load(Ordering::Relaxed).saturating_add(size) > self.max_bytes {
            return false;
        }
        if fs_available_bytes(&self.dir) < self.min_free {
            return false;
        }

        let id   = self.next_id.fetch_add(1, Ordering::Relaxed);
        let path = self.dir.join(format!("{id:016x}.bin"));

        if let Err(e) = std::fs::write(&path, data) {
            warn!("disk_buffer write failed: {e}");
            return false;
        }
        self.used_bytes.fetch_add(size, Ordering::Relaxed);
        true
    }

    /// Return all buffer file paths sorted by name (= insertion order).
    pub fn drain_ordered(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&self.dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".bin"))
            .map(|e| e.path())
            .collect();
        paths.sort_unstable();
        paths
    }

    /// Remove a successfully-uploaded buffer file and update the counter.
    pub fn remove(&self, path: &Path) {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let _ = std::fs::remove_file(path);
        self.used_bytes.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |cur| Some(cur.saturating_sub(size)),
        ).ok();
    }

    pub fn used_bytes(&self) -> u64 {
        self.used_bytes.load(Ordering::Relaxed)
    }
}

#[cfg(unix)]
fn fs_available_bytes(path: &Path) -> u64 {
    use std::os::unix::ffi::OsStrExt;
    let p = match std::ffi::CString::new(path.as_os_str().as_bytes()) {
        Ok(p)  => p,
        Err(_) => return u64::MAX,
    };
    let mut sv: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(p.as_ptr(), &mut sv) } == 0 {
        (sv.f_bavail as u64).saturating_mul(sv.f_frsize as u64)
    } else {
        u64::MAX
    }
}

#[cfg(not(unix))]
fn fs_available_bytes(_path: &Path) -> u64 {
    u64::MAX
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_buf(dir: &TempDir, max: u64) -> DiskBuffer {
        DiskBuffer::new(dir.path().to_path_buf(), max, 0)
    }

    #[test]
    fn write_and_drain() {
        let dir = TempDir::new().unwrap();
        let buf = make_buf(&dir, 10 * 1024 * 1024);
        assert!(buf.try_write(b"hello world"));
        let files = buf.drain_ordered();
        assert_eq!(files.len(), 1);
        assert_eq!(std::fs::read(&files[0]).unwrap(), b"hello world");
    }

    #[test]
    fn cap_at_max_bytes() {
        let dir  = TempDir::new().unwrap();
        let buf  = make_buf(&dir, 100);
        let mut written = 0usize;
        for _ in 0..20 {
            if buf.try_write(&[0u8; 20]) { written += 1; }
        }
        // 20-byte writes; 100-byte cap → at most 5 succeed.
        assert!(written <= 5, "wrote {written} batches past cap");
    }

    #[test]
    fn remove_decrements_used_bytes() {
        let dir   = TempDir::new().unwrap();
        let buf   = make_buf(&dir, 10 * 1024 * 1024);
        buf.try_write(b"some data");
        let before = buf.used_bytes();
        let files  = buf.drain_ordered();
        buf.remove(&files[0]);
        assert!(buf.used_bytes() < before);
        assert!(buf.drain_ordered().is_empty());
    }

    #[test]
    fn never_panics_on_corrupt_file() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("0000000000000000.bin"), b"garbage").unwrap();
        // Construction with corrupt file must not panic.
        let buf   = DiskBuffer::new(dir.path().to_path_buf(), 10 * 1024 * 1024, 0);
        let files = buf.drain_ordered();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn persists_across_restart() {
        let dir = TempDir::new().unwrap();
        {
            let buf = make_buf(&dir, 10 * 1024 * 1024);
            buf.try_write(b"file1");
            buf.try_write(b"file2");
        }
        let buf2 = make_buf(&dir, 10 * 1024 * 1024);
        assert_eq!(buf2.drain_ordered().len(), 2);
        assert!(buf2.used_bytes() > 0);
    }

    #[test]
    fn insertion_order_preserved() {
        let dir = TempDir::new().unwrap();
        let buf = make_buf(&dir, 10 * 1024 * 1024);
        for i in 0u8..5 {
            buf.try_write(&[i; 10]);
        }
        let files = buf.drain_ordered();
        assert_eq!(files.len(), 5);
        for (i, path) in files.iter().enumerate() {
            let data = std::fs::read(path).unwrap();
            assert!(data.iter().all(|&b| b == i as u8));
        }
    }
}
