use crate::{daemon::Daemon, utilities};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Clone)]
pub struct AuthorizationTask {
    daemon: Arc<Mutex<Daemon>>,
    shutdown: broadcast::Sender<()>,
    timestamp: u64,
}

impl AuthorizationTask {
    pub fn new(daemon: Arc<Mutex<Daemon>>) -> Self {
        let timestamp = utilities::time::timestamp();
        let (shutdown, _) = broadcast::channel(1);
        Self {
            daemon,
            shutdown,
            timestamp: timestamp as u64,
        }
    }

    pub fn run(&self) {
        let mut receiver = self.shutdown.subscribe();
        let daemon = self.daemon.clone();

        tokio::spawn(async move {
            tokio::select! {
                _ = receiver.recv() => {}
                _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                    Daemon::on_authorized(daemon).await;
                }
            }
        });
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown.send(());
    }
}
