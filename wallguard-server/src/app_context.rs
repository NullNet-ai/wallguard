use nullnet_liberror::Error;

use crate::datastore::Datastore;
use crate::orchestrator::Orchestrator;

#[derive(Debug, Clone)]
pub struct AppContext {
    pub datastore: Datastore,
    pub orchestractor: Orchestrator,
}

impl AppContext {
    pub async fn new() -> Result<Self, Error> {
        let datastore = Datastore::new().await?;
        let orchestractor = Orchestrator::new();

        Ok(Self {
            datastore,
            orchestractor,
        })
    }
}
