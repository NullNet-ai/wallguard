use crate::datastore::{Datastore, db_tables::DBTable};
use nullnet_libdatastore::BatchCreateRequestBuilder;
use nullnet_liberror::Error;
use serde::Serialize;
use serde_json::json;
use wallguard_common::protobuf::{
    wallguard_models::{FilterRule, NatRule},
    wallguard_service::ConfigStatus,
};

impl Datastore {
    pub async fn create_filter_rules(
        &self,
        token: &str,
        rules: &[FilterRule],
        config_id: &str,
        status: ConfigStatus,
    ) -> Result<(), Error> {
        self.create_rules(token, rules, config_id, DBTable::DeviceFilterRules, status)
            .await
    }

    pub async fn create_nat_rules(
        &self,
        token: &str,
        rules: &[NatRule],
        config_id: &str,
        status: ConfigStatus,
    ) -> Result<(), Error> {
        self.create_rules(token, rules, config_id, DBTable::DeviceNatRules, status)
            .await
    }

    async fn create_rules<T: Serialize>(
        &self,
        token: &str,
        rules: &[T],
        config_id: &str,
        table: DBTable,
        status: ConfigStatus,
    ) -> Result<(), Error> {
        if rules.is_empty() {
            return Ok(());
        }

        let records: Vec<serde_json::Value> = rules
            .iter()
            .map(|record| {
                let mut json = serde_json::to_value(record).expect("Serialization failed");
                json["device_configuration_id"] = json!(config_id);

                json["device_rule_status"] = match status {
                    ConfigStatus::CsDraft => json!("Draft"),
                    ConfigStatus::CsApplied => json!("Applied"),
                    ConfigStatus::CsUndefined => json!("Undefined"),
                };

                json
            })
            .collect();

        let request = BatchCreateRequestBuilder::new()
            .table(table)
            .durability("hard")
            .entity_prefix("RL")
            .records(serde_json::to_string(&serde_json::Value::Array(records)).unwrap())
            .build();

        self.inner.clone().batch_create(request, token).await?;

        Ok(())
    }
}
