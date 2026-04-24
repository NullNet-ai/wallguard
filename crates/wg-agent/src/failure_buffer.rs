use std::collections::VecDeque;
use std::io::{BufWriter, Write as IoWrite};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use uuid::Uuid;
use wg_shared::types::{FailureCategory, FailureSeverity};

const MAX_ENTRIES: usize = 500;

/// Process-global failure buffer. Initialised once in `main` before the
/// tokio runtime starts, so the panic hook can access it without async.
pub static BUFFER: OnceLock<FailureBuffer> = OnceLock::new();

// ---------------------------------------------------------------------------
// Entry type — serialised to / from NDJSON on disk
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FailureEntry {
    pub failure_id:  Uuid,
    pub severity:    FailureSeverity,
    pub category:    FailureCategory,
    pub message:     String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context:     Option<String>,
    pub occurred_at: u64,   // Unix ms at time of occurrence
    #[serde(default)]
    pub is_replay:   bool,
}

// ---------------------------------------------------------------------------
// Buffer
// ---------------------------------------------------------------------------

pub struct FailureBuffer {
    inner: Mutex<Inner>,
}

struct Inner {
    entries: VecDeque<FailureEntry>,
    path:    PathBuf,
}

impl FailureBuffer {
    /// Load existing entries from `path` (corrupt lines are silently skipped)
    /// or start empty if the file does not exist.
    pub fn load_or_create(path: PathBuf) -> Self {
        let entries = load_entries(&path);
        Self { inner: Mutex::new(Inner { entries, path }) }
    }

    /// Append an entry synchronously. Safe to call from a panic hook — uses
    /// `Mutex::lock()` which blocks but never panics on a poisoned lock.
    pub fn append_sync(&self, entry: FailureEntry) {
        let is_fatal = entry.severity == FailureSeverity::Fatal;
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(entry);
        if let Err(e) = guard.write_to_disk(is_fatal) {
            eprintln!("failure_buffer: disk write error: {e}");
        }
    }

    /// Async wrapper — delegates to the sync path.
    pub async fn append(&self, entry: FailureEntry) {
        self.append_sync(entry);
    }

    /// Returns all buffered entries in insertion order.
    pub fn read_all(&self) -> Vec<FailureEntry> {
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        guard.entries.iter().cloned().collect()
    }

    /// Remove entries whose `failure_id` matches a delivered ID and rewrite
    /// the on-disk file.
    pub fn trim_delivered(&self, delivered_ids: &[Uuid]) {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        guard.entries.retain(|e| !delivered_ids.contains(&e.failure_id));
        if let Err(e) = guard.write_to_disk(false) {
            eprintln!("failure_buffer: trim rewrite error: {e}");
        }
    }

    /// Non-blocking append for use from the panic hook. Returns `true` if the
    /// lock was acquired and the entry was appended.
    pub fn try_append_sync(&self, entry: FailureEntry) -> bool {
        let is_fatal = entry.severity == FailureSeverity::Fatal;
        let mut guard = match self.inner.try_lock() {
            Ok(g)  => g,
            Err(_) => return false,
        };
        guard.push(entry);
        let _ = guard.write_to_disk(is_fatal);
        true
    }
}

impl Inner {
    fn push(&mut self, entry: FailureEntry) {
        if self.entries.len() >= MAX_ENTRIES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    fn write_to_disk(&self, fsync: bool) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;
        let mut w = BufWriter::new(file);

        for entry in &self.entries {
            if let Ok(line) = serde_json::to_string(entry) {
                w.write_all(line.as_bytes())?;
                w.write_all(b"\n")?;
            }
        }
        w.flush()?;

        if fsync {
            let f = w.into_inner().map_err(|e| e.into_error())?;
            f.sync_data()?;
        }

        Ok(())
    }
}

fn load_entries(path: &PathBuf) -> VecDeque<FailureEntry> {
    let content = match std::fs::read_to_string(path) {
        Ok(s)  => s,
        Err(_) => return VecDeque::new(),
    };

    let mut entries = VecDeque::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if let Ok(e) = serde_json::from_str::<FailureEntry>(line) {
            entries.push_back(e);
        }
        // silently skip corrupt lines
    }

    while entries.len() > MAX_ENTRIES {
        entries.pop_front();
    }

    entries
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wg_shared::types::{FailureCategory, FailureSeverity};

    fn entry(msg: &str) -> FailureEntry {
        FailureEntry {
            failure_id:  Uuid::new_v4(),
            severity:    FailureSeverity::Warning,
            category:    FailureCategory::System,
            message:     msg.to_string(),
            context:     None,
            occurred_at: 0,
            is_replay:   false,
        }
    }

    fn buf_at(dir: &tempfile::TempDir) -> FailureBuffer {
        FailureBuffer::load_or_create(dir.path().join("failures.jsonl"))
    }

    #[test]
    fn cap_at_exactly_500() {
        let dir = tempfile::tempdir().unwrap();
        let buf = buf_at(&dir);

        for i in 0..=500 {
            buf.append_sync(entry(&format!("msg {i}")));
        }

        let all = buf.read_all();
        assert_eq!(all.len(), MAX_ENTRIES, "buffer must cap at {MAX_ENTRIES}");
        assert_eq!(all[0].message, "msg 1", "oldest entry should be msg 1 (msg 0 was evicted)");
        assert_eq!(all[MAX_ENTRIES - 1].message, "msg 500");
    }

    #[test]
    fn trim_delivered_removes_correct_entries() {
        let dir = tempfile::tempdir().unwrap();
        let buf = buf_at(&dir);

        let entries: Vec<FailureEntry> = (0..5).map(|i| entry(&format!("e{i}"))).collect();
        let ids_to_trim = vec![entries[1].failure_id, entries[3].failure_id];

        for e in &entries {
            buf.append_sync(e.clone());
        }

        buf.trim_delivered(&ids_to_trim);

        let remaining = buf.read_all();
        assert_eq!(remaining.len(), 3);
        let msgs: Vec<&str> = remaining.iter().map(|e| e.message.as_str()).collect();
        assert!(msgs.contains(&"e0"));
        assert!(msgs.contains(&"e2"));
        assert!(msgs.contains(&"e4"));
        assert!(!msgs.contains(&"e1"));
        assert!(!msgs.contains(&"e3"));
    }

    #[test]
    fn never_panics_on_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("failures.jsonl");
        std::fs::write(&path, b"not json\n{\"broken\": }\n").unwrap();

        // Must not panic.
        let buf = FailureBuffer::load_or_create(path);
        assert_eq!(buf.read_all().len(), 0, "corrupt lines should be skipped");
    }

    #[test]
    fn persists_and_reloads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("failures.jsonl");

        {
            let buf = FailureBuffer::load_or_create(path.clone());
            buf.append_sync(entry("persistent"));
        }

        let buf2 = FailureBuffer::load_or_create(path);
        let all  = buf2.read_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].message, "persistent");
    }
}
