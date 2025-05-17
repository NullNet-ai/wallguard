use nullnet_liberror::Error;

use crate::datastore::Datastore;
use crate::orchestrator::Orchestrator;
use crate::reverse_tunnel::ReverseTunnel;

#[derive(Debug, Clone)]
pub struct AppContext {
    pub datastore: Datastore,
    pub orchestractor: Orchestrator,
    pub tunnel: ReverseTunnel,
}

impl AppContext {
    pub async fn new() -> Result<Self, Error> {
        let datastore = Datastore::new().await?;
        let orchestractor = Orchestrator::new();

        let tunnel = ReverseTunnel::new();

        Ok(Self {
            datastore,
            orchestractor,
            tunnel,
        })
    }
}
