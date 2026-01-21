use crate::datastore::Datastore;
use crate::datastore::db_tables::DBTable;
use nullnet_libdatastore::{AdvanceFilterBuilder, BatchUpdateRequestBuilder};
use nullnet_liberror::Error;
use serde_json::json;
use wallguard_common::protobuf::wallguard_service::ConfigStatus;

impl Datastore {
    pub async fn update_rules_status(
        &self,
        token: &str,
        config_id: &str,
        status: ConfigStatus,
    ) -> Result<(), Error> {
        let (r1, r2) = tokio::join!(
            self.update_rules_status_internal(token, config_id, status, DBTable::DeviceFilterRules),
            self.update_rules_status_internal(token, config_id, status, DBTable::DeviceNatRules),
        );

        r1?;
        r2?;

        Ok(())
    }

    async fn update_rules_status_internal(
        &self,
        token: &str,
        config_id: &str,
        status: ConfigStatus,
        table: DBTable,
    ) -> Result<(), Error> {
        let updates = json!({"device_rule_status": match status {
            ConfigStatus::CsDraft => "Draft",
            ConfigStatus::CsApplied => "Applied",
            ConfigStatus::CsUndefined => "Undefined",
        }});

        let filter = AdvanceFilterBuilder::new()
            .field("device_configuration_id")
            .values(format!("[\"{config_id}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(table)
            .build();

        let request = BatchUpdateRequestBuilder::new()
            .advance_filter(filter)
            .performed_by_root(false)
            .table(table)
            .updates(updates.to_string())
            .build();

        let _ = self.inner.clone().batch_update(request, token).await?;

        Ok(())
    }
}
