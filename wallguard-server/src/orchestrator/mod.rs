use std::sync::Arc;
use tokio::sync::Mutex;

mod client;
mod control_channel;

#[derive(Debug, Default)]
struct OrchestratorInner;

#[derive(Debug, Clone, Default)]
pub struct Orchestrator {
    inner: Arc<Mutex<OrchestratorInner>>,
}


impl Orchestrator {
    pub async fn is_client_connected(&self, device_id: &str) -> bool {
        todo!()
    }
}