use nullnet_libdatastore::DeleteRequestBuilder;
use nullnet_liberror::Error;

use crate::datastore::{Datastore, RemoteAccessSession};

impl Datastore {
    pub async fn delete_remote_access_session(
        &self,
        token: &str,
        session_id: &str,
    ) -> Result<(), Error> {
        let request = DeleteRequestBuilder::new()
            .id(session_id)
            .table(RemoteAccessSession::table())
            .build();

        let _ = self.inner.clone().delete(request, token).await?;

        Ok(())
    }
}
