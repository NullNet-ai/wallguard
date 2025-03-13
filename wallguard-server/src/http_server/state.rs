use crate::tunnel::TunnelServer;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct State {
    pub tunnel: Arc<Mutex<TunnelServer>>,
}
