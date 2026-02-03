use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::http_api::ssh_gateway_v2::session::Session;

#[derive(Debug, Default, Clone)]
pub struct SshSessionsManager {
    sessions: Arc<Mutex<HashMap<String, Arc<Mutex<Session>>>>>,
}

impl SshSessionsManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn add(&self, id: String, session: Arc<Mutex<Session>>) {
        let mut sessions = self.sessions.lock().await;
        sessions.insert(id, session);
    }

    pub async fn remove(&self, id: &str) -> Option<Arc<Mutex<Session>>> {
        let mut sessions = self.sessions.lock().await;
        sessions.remove(id)
    }

    pub async fn get_by_id(&self, id: &str) -> Option<Arc<Mutex<Session>>> {
        let sessions = self.sessions.lock().await;
        sessions.get(id).cloned()
    }
}
