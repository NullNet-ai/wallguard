use client::Instance;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    app_context::AppContext,
    orchestrator::{
        client::{InboundStream, OutboundStream},
        new_connection_handler::NewConnectionHandler,
    },
};

mod auth_request_handler;
mod client;
mod control_stream;
mod new_connection_handler;

type InstancesVector = Arc<Mutex<Vec<Arc<Mutex<Instance>>>>>;
type ClientsMap = Arc<Mutex<HashMap<String, InstancesVector>>>;

#[derive(Debug, Clone, Default)]
pub struct Orchestrator {
    pub(crate) clients: ClientsMap,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_new_connection(
        &self,
        inbound: InboundStream,
        outbound: OutboundStream,
        context: AppContext,
    ) {
        log::info!("Orchestrator: on_new_connection");
        let handler = NewConnectionHandler::new(context);
        tokio::spawn(handler.handle(inbound, outbound));
    }

    pub async fn on_disconnected(&self, device_id: &str, instance_id: &str) -> Result<(), Error> {
        log::info!("Orchestrator: on_client_disconnected, id {device_id}, Instance {instance_id}");

        let lock = self.clients.lock().await;

        if lock.get(device_id).is_none() {
            Err(format!("Device with ID '{device_id}' is not connected"))
                .handle_err(location!())?;
        }

        let mut instances = lock.get(device_id).unwrap().lock().await;

        let filtered: Vec<_> = instances
            .drain(..)
            .filter(|instance| match instance.try_lock() {
                Ok(inst) => inst.instance_id != instance_id,
                Err(_) => true,
            })
            .collect();

        *instances = filtered;

        Ok(())
    }

    pub async fn get_client_instances(&self, device_id: &str) -> Option<InstancesVector> {
        self.clients.lock().await.get(device_id).cloned()
    }

    pub async fn get_any_client_instance(&self, device_id: &str) -> Option<Arc<Mutex<Instance>>> {
        self.get_client_instances(device_id)
            .await?
            .lock()
            .await
            .first()
            .cloned()
    }

    pub async fn get_client(
        &self,
        device_id: &str,
        instance_id: &str,
    ) -> Option<Arc<Mutex<Instance>>> {
        let instances = self.get_client_instances(device_id).await?;

        for instance in instances.lock().await.iter() {
            if instance.lock().await.instance_id == instance_id {
                return Some(instance.clone());
            }
        }

        None
    }

    pub async fn does_client_have_connected_instances(&self, device_id: &str) -> bool {
        if let Some(vec) = self.clients.lock().await.get(device_id) {
            !vec.lock().await.is_empty()
        } else {
            false
        }
    }
}
