use crate::datastore::{Datastore, RemoteAccessSession};
use nullnet_libdatastore::CreateRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_session(
        &self,
        token: &str,
        session: &RemoteAccessSession,
    ) -> Result<(), Error> {
        let mut json = serde_json::to_value(session).handle_err(location!())?;
        json.as_object_mut().unwrap().remove("id");

        let request = CreateRequestBuilder::new()
            .pluck(RemoteAccessSession::pluck())
            .table(RemoteAccessSession::table())
            .record(json.to_string())
            .build();

        let _ = self.inner.clone().create(request, token).await?;
        Ok(())
    }
}
