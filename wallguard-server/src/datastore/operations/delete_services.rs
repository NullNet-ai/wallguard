use crate::datastore::{Datastore, ServiceInfo, db_tables::DBTable};
use nullnet_libdatastore::{AdvanceFilterBuilder, BatchDeleteRequestBuilder};
use nullnet_liberror::Error;

impl Datastore {
    pub async fn delete_services(
        &self,
        token: &str,
        services: &[ServiceInfo],
    ) -> Result<(), Error> {
        if services.is_empty() {
            return Ok(());
        }

        let values = services
            .iter()
            .map(|svc| format!("\"{}\"", svc.id.clone()))
            .collect::<Vec<_>>()
            .join(",");

        let filter = AdvanceFilterBuilder::new()
            .field("id")
            .values(format!("[{values}]"))
            .r#type("criteria")
            .operator("contains")
            .entity(DBTable::DeviceServices)
            .build();

        let request = BatchDeleteRequestBuilder::new()
            .table(DBTable::DeviceServices)
            .advance_filter(filter)
            .build();

        self.inner.clone().batch_delete(request, token).await?;

        Ok(())
    }
}
