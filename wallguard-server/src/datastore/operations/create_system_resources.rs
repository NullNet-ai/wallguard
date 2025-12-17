use crate::datastore::Datastore;
use crate::datastore::db_tables::DBTable;
use nullnet_libdatastore::BatchCreateRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde_json::json;
use wallguard_common::protobuf::wallguard_service::SystemResource;

impl Datastore {
    pub async fn create_system_resources(
        &self,
        token: &str,
        resources: Vec<SystemResource>,
        device_id: String,
    ) -> Result<(), Error> {
        // Inject the device ID manually.
        // TODO: Move this logic to the agent.
        // The agent should use `libtoken` to parse the JWT and extract the device ID.
        let mapped_values: Vec<serde_json::Value> = resources
            .iter()
            .map(|res| {
                let mut json = json!(res);
                json["device_id"] = json!(device_id);
                json
            })
            .collect();

        let records = serde_json::to_string(&mapped_values).handle_err(location!())?;

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
