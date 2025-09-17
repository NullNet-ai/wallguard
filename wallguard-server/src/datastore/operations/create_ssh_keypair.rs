use nullnet_liberror::Error;
use serde_json::json;

use crate::datastore::{Datastore, SSHKeypair};
use nullnet_libdatastore::CreateRequestBuilder;

impl Datastore {
    pub async fn create_ssh_keypair(&self, token: &str, keypair: &SSHKeypair) -> Result<(), Error> {
        let request = CreateRequestBuilder::new()
            .pluck(SSHKeypair::pluck())
            .table(SSHKeypair::table())
            .record(json!(keypair).to_string())
            .build();

        let _ = self.inner.clone().create(request, token).await?;
        Ok(())
    }
}
