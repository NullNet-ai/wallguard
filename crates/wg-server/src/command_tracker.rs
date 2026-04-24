use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{oneshot, Mutex};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const SWEEP_INTERVAL: Duration  = Duration::from_secs(5);

// ---------------------------------------------------------------------------
// Outcome returned to HTTP handlers
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum CommandOutcome {
    /// Agent reported success; `output` holds stdout for named commands.
    Success { output: String, applied_digest: String },
    /// Agent reported failure.
    Failure { error_message: String },
    /// Command timed out (30s without a CommandResult).
    Timeout,
}

// ---------------------------------------------------------------------------
// Tracker
// ---------------------------------------------------------------------------

struct Pending {
    tx:      oneshot::Sender<CommandOutcome>,
    sent_at: Instant,
}

#[derive(Clone)]
pub struct CommandTracker {
    inner: Arc<Mutex<HashMap<String, Pending>>>,
}

impl CommandTracker {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Register a pending command and return a receiver that resolves when the
    /// agent sends back a `CommandResult` or the sweeper times it out.
    pub async fn register(&self, command_id: &str) -> oneshot::Receiver<CommandOutcome> {
        let (tx, rx) = oneshot::channel();
        self.inner.lock().await.insert(
            command_id.to_string(),
            Pending { tx, sent_at: Instant::now() },
        );
        rx
    }

    /// Resolve a pending command with a result from the agent.
    pub async fn resolve(&self, command_id: &str, outcome: CommandOutcome) {
        if let Some(p) = self.inner.lock().await.remove(command_id) {
            let _ = p.tx.send(outcome);
        }
    }

    /// Immediately time out all pending commands (for graceful shutdown).
    pub async fn timeout_all(&self) {
        for (_, p) in self.inner.lock().await.drain() {
            let _ = p.tx.send(CommandOutcome::Timeout);
        }
    }

    /// Spawn the background sweeper that times out stale commands.
    /// The returned handle can be aborted on shutdown.
    pub fn start_sweeper(&self) -> tokio::task::JoinHandle<()> {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(SWEEP_INTERVAL);
            loop {
                interval.tick().await;
                let now  = Instant::now();
                let mut map = inner.lock().await;
                let timed_out: Vec<String> = map
                    .iter()
                    .filter(|(_, p)| now.duration_since(p.sent_at) >= COMMAND_TIMEOUT)
                    .map(|(id, _)| id.clone())
                    .collect();
                for id in timed_out {
                    if let Some(p) = map.remove(&id) {
                        tracing::warn!(command_id = %id, "command timed out");
                        let _ = p.tx.send(CommandOutcome::Timeout);
                    }
                }
            }
        })
    }
}
