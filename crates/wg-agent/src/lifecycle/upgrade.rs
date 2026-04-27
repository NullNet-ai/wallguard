use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use tokio::time::timeout;
use tracing::{info, warn};

static IN_FLIGHT: std::sync::OnceLock<Arc<AtomicI32>> = std::sync::OnceLock::new();

fn counter() -> &'static Arc<AtomicI32> {
    IN_FLIGHT.get_or_init(|| Arc::new(AtomicI32::new(0)))
}

/// RAII guard that increments the in-flight counter on creation and
/// decrements it on drop. Wrap each tunnel/command task body with this
/// so `drain()` can wait for them to finish.
pub struct InFlightGuard;

impl InFlightGuard {
    pub fn new() -> Self {
        counter().fetch_add(1, Ordering::Relaxed);
        InFlightGuard
    }
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        counter().fetch_sub(1, Ordering::Relaxed);
    }
}

/// Wait until all in-flight command/tunnel guards are dropped, or until
/// `drain_timeout_ms` elapses (0 → 10 000 ms default).
pub async fn drain(drain_timeout_ms: u32) {
    let ms      = if drain_timeout_ms == 0 { 10_000 } else { drain_timeout_ms as u64 };
    let deadline = Duration::from_millis(ms);

    let result = timeout(deadline, async {
        loop {
            if counter().load(Ordering::Relaxed) <= 0 {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;

    match result {
        Ok(_)  => info!("graceful restart: in-flight commands drained"),
        Err(_) => warn!("graceful restart: drain timeout ({ms} ms) — forcing shutdown"),
    }
}
