use crate::datastore::{Datastore, ServiceInfo, db_tables::DBTable};
use nullnet_libdatastore::{
    AdvanceFilterBuilder, BatchCreateRequestBuilder, BatchDeleteRequestBuilder,
};
use nullnet_liberror::Error;

impl Datastore {
    pub async fn udpate_services(
        &self,
        token: &str,
        device_id: &str,
        services: &[ServiceInfo],
    ) -> Result<(), Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("device_id")
            .values(format!("[\"{device_id}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(DBTable::DeviceServices)
            .build();

        let request = BatchDeleteRequestBuilder::new()
            .table(DBTable::DeviceServices)
            .advance_filter(filter)
            .build();

        self.inner.clone().batch_delete(request, token).await?;

        if services.is_empty() {
            return Ok(());
        }

        let records: Vec<serde_json::Value> = services
            .iter()
            .map(|record| serde_json::to_value(record).unwrap())
            .collect();

        let request = BatchCreateRequestBuilder::new()
            .table(DBTable::DeviceServices)
            .durability("hard")
            .entity_prefix("SI")
            .records(serde_json::to_string(&serde_json::Value::Array(records)).unwrap())
            .build();

        self.inner.clone().batch_create(request, token).await?;

        Ok(())
    }
}
