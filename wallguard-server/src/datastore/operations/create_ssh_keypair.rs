use nullnet_liberror::Error;
use serde_json::json;

use crate::datastore::builders::CreateRequestBuilder;
use crate::datastore::{Datastore, SSHKeypair};

impl Datastore {
    pub async fn create_ssh_keypair(&self, token: &str, keypair: &SSHKeypair) -> Result<(), Error> {
        let request = CreateRequestBuilder::new()
            .pluck(SSHKeypair::pluck())
            .table(SSHKeypair::table())
            .record(json!(keypair).to_string())
            .entity_prefix(SSHKeypair::entity_prefix())
            .build();

        let _ = self.inner.clone().create(request, token).await?;
        Ok(())
    }
}
