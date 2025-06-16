use crate::datastore::{Datastore, builders::BatchCreateRequestBuilder, db_tables::DBTable};
use libfireparse::Rule;
use nullnet_liberror::Error;
use serde_json::json;

impl Datastore {
    pub async fn create_rules(
        &self,
        token: &str,
        rules: &Vec<Rule>,
        config_id: &str,
    ) -> Result<(), Error> {
        if rules.is_empty() {
            return Ok(());
        }

        let records: Vec<serde_json::Value> = rules
            .iter()
            .map(|record| {
                let mut json = serde_json::to_value(record).expect("Serialization failed");
                json["device_configuration_id"] = json!(config_id);
                json
            })
            .collect();

        let request = BatchCreateRequestBuilder::new()
            .table(DBTable::DeviceRules)
            .durability("hard")
            .entity_prefix("RL")
            .records(serde_json::to_string(&serde_json::Value::Array(records)).unwrap())
            .build();

        let _ = self.inner.clone().batch_create(request, token).await?;

        Ok(())
    }
}
