use crate::{
    app_context::AppContext,
    tunneling::{
        http::HttpTunnel,
        ssh::SshTunnel,
        tty::TtyTunnel,
        tunnel_common::{TunnelCommonData, TunnelCreateError, WallguardTunnel},
    },
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

mod command;
pub mod http;
pub mod ssh;
pub mod tty;
pub mod tunnel_common;

#[derive(Debug, Clone)]
pub struct TunnelsManager {
    tunnels: Arc<Mutex<HashMap<String, WallguardTunnel>>>,
}

impl TunnelsManager {
    pub fn new() -> Self {
        Self {
            tunnels: Default::default(),
        }
    }

    pub async fn request(
        &self,
        jwt: &str,
        device_id: &str,
        service_id: &str,
        context: Arc<AppContext>,
    ) -> Result<String, TunnelCreateError> {
        use crate::datastore::TunnelType;

        let data = TunnelCommonData::create(context.clone(), jwt, device_id, service_id).await?;

        let tunnel_id = data.tunnel_data.id.clone();

        let tunnel = match data.tunnel_data.tunnel_type {
            TunnelType::Http | TunnelType::Https => {
                let tunnel = HttpTunnel::new(context.clone(), data);
                WallguardTunnel::Http(Arc::new(Mutex::new(tunnel)))
            }
            TunnelType::Ssh => {
                let tunnel = SshTunnel::new(context.clone(), data).await?;
                WallguardTunnel::Ssh(Arc::new(Mutex::new(tunnel)))
            }
            TunnelType::Tty => {
                let tunnel = TtyTunnel::new(context.clone(), data).await?;
                WallguardTunnel::Tty(Arc::new(Mutex::new(tunnel)))
            }
        };

        self.tunnels.lock().await.insert(tunnel_id.clone(), tunnel);

        Ok(tunnel_id)
    }

    pub async fn get(&self, tunnel_id: &str) -> Option<WallguardTunnel> {
        self.tunnels.lock().await.get(tunnel_id).cloned()
    }

    pub async fn on_tunnel_terminated(&self, tunnel_id: &str) {
        if let Some(tunnel) = self.tunnels.lock().await.remove(tunnel_id) {
            let _ = tunnel.terminate().await;
        }
    }
}
