//! OS-level advisory locking used to guarantee that only one copy of the
//! WallGuard agent is ever running at a time, regardless of how it was
//! started (direct invocation, `wallguard-cli start`, systemd, launchd,
//! an rc.d script, or a Windows service).
//!
//! Both the agent and the CLI resolve the same lock file path so the CLI
//! can check agent liveness without depending on the agent crate.

use fs4::FileExt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Directory used for WallGuard's machine-wide runtime state (config, lock
/// file). The daemon always runs as root/Administrator, so this is safe to
/// keep out of any single user's home directory.
pub fn state_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        // On macOS the root user's home is /var/root, not /root.
        PathBuf::from("/var/root/.config/wallguard")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        PathBuf::from("/root/.config/wallguard")
    }
    #[cfg(windows)]
    {
        let base = std::env::var("PROGRAMDATA").unwrap_or_else(|_| r"C:\ProgramData".to_string());
        PathBuf::from(base).join("wallguard")
    }
}

/// Path to the lock file that guards single-instance enforcement for the
/// WallGuard agent.
pub fn agent_lock_path() -> PathBuf {
    state_dir().join("wallguard.lock")
}

/// Holds an exclusive OS-level advisory lock for as long as it lives.
///
/// The lock is released automatically when this value is dropped, including
/// on crash or `SIGKILL`, since the OS releases the underlying `flock`
/// (Unix) / `LockFileEx` (Windows) lock as soon as the file descriptor
/// closes — no manual cleanup is required.
pub struct InstanceLock(#[allow(dead_code)] File);

impl InstanceLock {
    /// Tries to become the sole holder of the lock at `path`, creating the
    /// file (and any missing parent directories) if needed.
    ///
    /// Returns `Ok(None)` if another live process already holds the lock.
    pub fn try_acquire(path: &Path) -> io::Result<Option<Self>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            // Never truncate on open: the file may currently be locked by a
            // live process, and we only want to clear its contents (the PID
            // we write below) after we've actually won the lock ourselves.
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;

        match FileExt::try_lock(&file) {
            Ok(()) => {
                // Best-effort: record the owning PID so ops can inspect the
                // file to see who is holding it (`cat` the lock path).
                let _ = file.set_len(0);
                let _ = file.seek(SeekFrom::Start(0));
                let _ = write!(file, "{}", std::process::id());
                Ok(Some(Self(file)))
            }
            Err(fs4::TryLockError::WouldBlock) => Ok(None),
            Err(fs4::TryLockError::Error(err)) => Err(err),
        }
    }
}
