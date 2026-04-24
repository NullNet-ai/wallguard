use uuid::Uuid;
use wg_shared::types::{FailureCategory, FailureSeverity};

use crate::failure_buffer::FailureEntry;

/// Install the global panic hook.
///
/// The hook writes a FATAL `FailureEntry` to the process-global
/// `failure_buffer::BUFFER` (using `try_lock` to avoid deadlock), then
/// falls back to `eprintln!` if the lock cannot be acquired or if the
/// buffer was never initialised.
pub fn install() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{info}");
        eprintln!("FATAL: wg-agent panic: {msg}");

        if let Some(buf) = crate::failure_buffer::BUFFER.get() {
            let entry = FailureEntry {
                failure_id:  Uuid::new_v4(),
                severity:    FailureSeverity::Fatal,
                category:    FailureCategory::AgentCrash,
                message:     msg,
                context:     None,
                occurred_at: unix_ms_now(),
                is_replay:   false,
            };
            if !buf.try_append_sync(entry) {
                eprintln!("failure_buffer: could not acquire lock from panic hook");
            }
        }
    }));
}

fn unix_ms_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
