use crate::datastore::{Datastore, ServiceInfo, db_tables::DBTable};
use nullnet_libdatastore::BatchCreateRequestBuilder;
use nullnet_liberror::Error;

impl Datastore {
    pub async fn create_services(
        &self,
        token: &str,
        services: &[ServiceInfo],
    ) -> Result<(), Error> {
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
