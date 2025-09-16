use crate::datastore::{Datastore, DeviceConfiguration, db_tables::DBTable};
use nullnet_libdatastore::UpdateRequestBuilder;
use nullnet_liberror::Error;
use serde_json::json;

impl Datastore {
    pub async fn update_config(
        &self,
        token: &str,
        config_id: &str,
        config: &DeviceConfiguration,
    ) -> Result<(), Error> {
        let request = UpdateRequestBuilder::new()
            .id(config_id)
            .table(DBTable::DeviceConfigurations)
            .body(json!(config).to_string())
            .build();

        let _ = self.inner.clone().update(request, token).await?;

        Ok(())
    }
}
