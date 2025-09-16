use nullnet_liberror::Error;
use serde_json::json;

use nullnet_libdatastore::CreateRequestBuilder;
use crate::datastore::{Datastore, RemoteAccessSession};

impl Datastore {
    pub async fn create_session(
        &self,
        token: &str,
        session: &RemoteAccessSession,
    ) -> Result<(), Error> {
        let request = CreateRequestBuilder::new()
            .pluck(RemoteAccessSession::pluck())
            .table(RemoteAccessSession::table())
            .record(json!(session).to_string())
            .build();

        let _ = self.inner.clone().create(request, token).await?;
        Ok(())
    }
}
