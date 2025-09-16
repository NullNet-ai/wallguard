use crate::datastore::Datastore;
use crate::datastore::db_tables::DBTable;
use nullnet_libdatastore::BatchCreateRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use wallguard_common::protobuf::wallguard_service::SystemResource;

impl Datastore {
    pub async fn create_system_resources(
        &self,
        token: &str,
        resources: Vec<SystemResource>,
    ) -> Result<(), Error> {
        let records = serde_json::to_string(&resources).handle_err(location!())?;

        let request = BatchCreateRequestBuilder::new()
            .table(DBTable::SystemResources)
            .entity_prefix("SR")
            .records(records)
            .build();

        self.inner
            .clone()
            .batch_create(request, token)
            .await
            .map(|_| ())
    }
}
