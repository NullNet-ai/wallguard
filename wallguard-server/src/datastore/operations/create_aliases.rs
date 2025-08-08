use crate::datastore::{Datastore, builders::BatchCreateRequestBuilder, db_tables::DBTable};
use nullnet_liberror::Error;
use serde_json::json;
use wallguard_common::protobuf::wallguard_models::Alias;

impl Datastore {
    pub async fn create_aliases(
        &self,
        token: &str,
        aliases: &[Alias],
        config_id: &str,
    ) -> Result<(), Error> {
        if aliases.is_empty() {
            return Ok(());
        }

        let records: Vec<serde_json::Value> = aliases
            .iter()
            .map(|record| {
                let mut json = serde_json::to_value(record).expect("Serialization failed");
                json["device_configuration_id"] = json!(config_id);
                json
            })
            .collect();

        let request = BatchCreateRequestBuilder::new()
            .table(DBTable::DeviceAliases)
            .durability("hard")
            .entity_prefix("AL")
            .records(serde_json::to_string(&serde_json::Value::Array(records)).unwrap())
            .build();

        let _ = self.inner.clone().batch_create(request, token).await?;

        Ok(())
    }
}
