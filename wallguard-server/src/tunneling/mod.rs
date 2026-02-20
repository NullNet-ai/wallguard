use crate::{
    app_context::AppContext,
    tunneling::{
        http::HttpTunnel,
        tunnel_common::TunnelCommon,
        tunnel_common_data::{TunnelCommonData, TunnelCreateError},
    },
};
use nullnet_liberror::Error;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

mod async_io;
mod command;
mod http;
mod ssh;
mod tty;
mod tunnel_common;
mod tunnel_common_data;

pub type ActiveTunnel = Arc<Mutex<Box<dyn TunnelCommon>>>;

#[derive(Debug, Clone)]
pub struct TunnelsManager {
    tunnels: Arc<Mutex<HashMap<String, ActiveTunnel>>>,
    context: AppContext,
}

impl TunnelsManager {
    pub fn new(context: AppContext) -> Self {
        Self {
            context,
            tunnels: Default::default(),
        }
    }

    pub async fn request(
        &self,
        jwt: &str,
        device_id: &str,
        service_id: &str,
    ) -> Result<String, TunnelCreateError> {
        use crate::datastore::TunnelType;

        let data =
            TunnelCommonData::create(self.context.clone(), jwt, device_id, service_id).await?;

        let tunnel_id = data.tunnel_data.id.clone();

        let result = match data.tunnel_data.tunnel_type {
            TunnelType::Http | TunnelType::Https => {
                HttpTunnel::create(self.context.clone(), data).await
            }
            TunnelType::Tty => todo!(),
            TunnelType::Ssh => todo!(),
        };

        let Ok(tunnel) = result else {
            let _ = self.context.datastore.delete_tunnel(jwt, &tunnel_id).await;
            return Err(TunnelCreateError::CanEstablishATunnel);
        };

        self.tunnels
            .lock()
            .await
            .insert(tunnel_id.clone(), Arc::new(Mutex::new(Box::new(tunnel))));

        Ok(tunnel_id)
    }

    pub async fn get(&self, tunnel_id: &str) -> Option<ActiveTunnel> {
        self.tunnels.lock().await.get(tunnel_id).cloned()
    }
}
